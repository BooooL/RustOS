// TODO(ryan): it really looks like bulk of libgreen could be used here where pthread <-> core

use core::prelude::*;
use core::cell::UnsafeCell;
use core::mem::{transmute, transmute_copy};

use alloc::boxed::Box;

use collections::LinkedList;

use thread::context::Context;
use thread::stack::Stack;

// thread control block
struct Tcb { 
  context: Context,
}

// invariant: current thread is at back of queue
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
  warn!("didn't unschedule finished thread");
  unreachable!();
}


impl Scheduler {
  
  pub fn new() -> Scheduler {
    Scheduler { queue: LinkedList::new() }
  }
  
  pub fn schedule(&mut self, func: Box<Fn() -> ()>) {
    let new_tcb = self.new_tcb(func);
    self.queue.push_front(new_tcb);
  }
  
  fn new_tcb(&self, func: Box<Fn() -> ()>) -> Tcb {
    const STACK_SIZE: usize = 1024 * 1024;
    let stack = Stack::new(STACK_SIZE);

    let p = move || {
      func();
      get_scheduler().unschedule_current();
    };
    
    let c = Context::new(run_thunk, box p as Box<Fn() -> ()>, stack);
    Tcb { context: c}
  }
  
  fn unschedule_current(&mut self) {
    let t = self.queue.pop_front().unwrap();
    debug!("unscheduling");
    let mut dont_care = Context::empty();
    Context::swap(&mut dont_care, &t.context);
  }
  
  pub fn switch(&mut self) {
    let next = self.queue.pop_front().unwrap();
    let old_context = &mut self.queue.back_mut().unwrap().context;
    Context::swap(old_context, &next.context);    
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
    s.schedule(box || { loop {} }); // this will be filled in with current context
    s.schedule(box t);
    debug!("schedule okay");
    s.switch();
    debug!("back");
  }
}
