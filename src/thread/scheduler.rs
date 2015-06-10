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
            trace!("in idle task 1");
            trace!("wait done");
            get_scheduler().switch();
            trace!("switch done");
            loop {}
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
  
  fn new_tcb(&self, func: Box<Fn() -> ()>) -> Tcb {
    const STACK_SIZE: usize = 1024 * 1024;
    let stack = Stack::new(STACK_SIZE);

    let p = move || {
      unsafe { cpu::current_cpu().enable_interrupts(); }
      func();
      get_scheduler().unschedule_current();
    };
    
    let c = Context::new(run_thunk, box p as Box<Fn() -> ()>, stack);
    Tcb { context: c }
  }
  
  pub fn schedule(&mut self, func: Box<Fn() -> ()>) {
    let new = self.new_tcb(func);
    self.schedule_tcb(new);    
  }
  
  fn schedule_tcb(&mut self, tcb: Tcb) {
    cpu::current_cpu().disable_interrupts();
    
    self.queue.push_back(tcb);
    
    cpu::current_cpu().enable_interrupts();
  }
  
  fn unschedule_current(&mut self) -> ! {
    let mut dont_care = Tcb { context: Context::empty() };
    self.do_and_unschedule(|_: Tcb| { &mut dont_care });
    unreachable!();    
  }
  
  fn do_and_unschedule<'a, F>(&mut self, mut do_something: F) where F : FnMut(Tcb) -> &'a mut Tcb {
    debug!("unscheduling");
    
    cpu::current_cpu().disable_interrupts();
        
    let save_into = do_something(self.queue.pop_front().unwrap()); // get rid of current
    let next = self.queue.pop_back().unwrap();
    self.queue.push_front(next);
    
    Context::swap(&mut save_into.context, &self.queue.front().unwrap().context);
    
    cpu::current_cpu().enable_interrupts();
  }
  
  pub fn switch(&mut self) {
    cpu::current_cpu().disable_interrupts();
    
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
    
    unsafe { cpu::current_cpu().enable_interrupts(); } // TODO(ryan): make a mutex as enabling/disabling interrupts
  }
  
}

struct Mutex {
    taken: bool,
    sleepers: LinkedList<Tcb>
}

impl Mutex {

    fn lock(&mut self) {
        cpu::current_cpu().disable_interrupts();
        while self.taken {
            get_scheduler().do_and_unschedule(|me: Tcb| { 
                self.sleepers.push_back(me);
                self.sleepers.back_mut().unwrap()
            });
        }
        self.taken = true;
        cpu::current_cpu().enable_interrupts();
    }
    
    fn try_lock(&mut self) -> bool {
        let mut ret;
        cpu::current_cpu().disable_interrupts();
        if self.taken {
            ret = false
        } else {
            self.taken = true;
            ret = true;
        }
        cpu::current_cpu().enable_interrupts();
        return ret;
    }
    
    fn unlock(&mut self) {
        cpu::current_cpu().disable_interrupts();
        assert!(self.taken);
        self.taken = false;
        match self.sleepers.pop_front() {
            Some(tcb) => get_scheduler().schedule_tcb(tcb),
            None => ()
        }
        cpu::current_cpu().enable_interrupts();
    }
    
    fn destroy(&mut self) {
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
