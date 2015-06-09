// TODO(ryan): it really looks like bulk of libgreen could be used here where pthread <-> core

use core::prelude::*;
use core::cell::UnsafeCell;
use core::mem::{transmute, transmute_copy};
use core::ptr;

use alloc::boxed::Box;

use collections::LinkedList;

use thread::context::Context;
use thread::stack::Stack;

use arch::cpu;

// thread control block
struct Tcb { 
  context: Context,
}

// invariant: current thread is at front of queue
pub struct Scheduler {
  queue: LinkedList<Tcb>
}

lazy_static! {
  static ref SCHEDULER: UnsafeCell<Scheduler> = UnsafeCell::new(Scheduler::new());
}

pub fn get_scheduler() -> &'static mut Scheduler {
    unsafe { transmute(SCHEDULER.get()) }
}

#[no_mangle]
extern "C" fn run_thunk(thunk: &Fn() -> ()) {
  debug!("in run_thunk");
  thunk();
  unreachable!("didn't unschedule finished thread");
}


impl Scheduler {
  
  pub fn new() -> Scheduler {
    let idle_task = || {
        loop {
            // trace!("in idle task");
            get_scheduler().switch();
        }
    };
    let mut s = Scheduler { queue: LinkedList::new() }; 
    let tcb = s.new_tcb(box idle_task);
    s.queue.push_front(tcb);
    s
  }
  
  pub fn bootstrap_start(&mut self) -> ! {
    // scheduler now takes control of the CPU
    // current context is discarded and front of queue is started
    let mut dont_care = Context::empty();
    Context::swap(&mut dont_care, &self.queue.front_mut().unwrap().context);
    unreachable!();
  }
  
  pub fn schedule(&mut self, func: Box<Fn() -> ()>) {
    let new_tcb = self.new_tcb(func);
    self.queue.push_back(new_tcb);
  }
  
  fn new_tcb(&self, func: Box<Fn() -> ()>) -> Tcb {
    const STACK_SIZE: usize = 1024 * 1024;
    let stack = Stack::new(STACK_SIZE);

    let p = move || {
      func();
      get_scheduler().unschedule_current();
    };
    
    let c = Context::new(run_thunk, box p as Box<Fn() -> ()>, stack);
    Tcb { context: c }
  }
  
  fn unschedule_current(&mut self) -> ! {
    debug!("unscheduling");
    
    self.queue.pop_front(); // get rid of current
    let next = self.queue.pop_back().unwrap();
    self.queue.push_front(next);
    
    let mut dont_care = Context::empty();
    Context::swap(&mut dont_care, &mut self.queue.front_mut().unwrap().context);
    unreachable!();
  }
  
  pub fn switch(&mut self) {
    if self.queue.len() == 1 {
        return;
    }
    let old = self.queue.pop_front().unwrap();
    let next = self.queue.pop_back().unwrap();
    self.queue.push_front(next);
    self.queue.push_back(old);
    
    let back: *mut Context = &mut self.queue.back_mut().unwrap().context;
    let front = self.queue.front().unwrap();
    Context::swap(unsafe { back.as_mut().unwrap() }, &front.context);    
  }
  
}

fn inner_thread_test(arg: usize) {
  debug!("arg is {}", arg)
}

extern "C" fn test_thread() {
  debug!("in a test thread!");
  inner_thread_test(11);
  unsafe {
    let s = get_scheduler();
    debug!("leaving test thread!"); 
    s.unschedule_current(); 
  }
}

pub fn thread_stuff() {
  debug!("starting thread test");
  unsafe {
    let s: &mut Scheduler = get_scheduler();

    debug!("orig sched 0x{:x}", transmute_copy::<_, u32>(&s));
    //loop {};
    let t = || { test_thread() };
    s.schedule(box t);
    debug!("schedule okay");
    s.switch();
    debug!("back");
  }
}
