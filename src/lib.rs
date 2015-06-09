#![no_std]


#![allow(improper_ctypes)]

#![feature(asm)]
#![feature(lang_items)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(core, alloc, collections)]
#![feature(no_std)]

// not directly used, but needed to link to llvm emitted calls
extern crate rlibc;

#[macro_use]
extern crate core;
extern crate alloc;
extern crate collections;

extern crate external as bump_ptr;
#[macro_use]
extern crate lazy_static;
extern crate spin;

use core::prelude::*;

use collections::Vec;
use ::io::Writer;
use multiboot::multiboot_info;
use arch::cpu;
use pci::Pci;
use driver::DriverManager;
use thread::scheduler;

#[macro_use]
mod log;
pub mod arch;
mod terminal;
mod panic;
mod multiboot;
//mod thread;
mod pci;
mod rtl8139;
mod driver;
mod net;
mod thread;

mod io;


fn test_allocator() {
  let mut v = Vec::new();

  debug!("Testing allocator with a vector push");
  v.push("   hello from a vector!");
  debug!("   push didn't crash");
  match v.pop() {
    Some(string) => debug!("{}", string),
    None => debug!("    push was weird...")
  }

}

fn put_char(c: u8) {
  __print!("{}", c as char);
}

#[no_mangle]
pub extern "C" fn main(magic: u32, info: *mut multiboot_info) -> ! {
    // some preliminaries
    bump_ptr::set_allocator((15usize * 1024 * 1024) as *mut u8, (20usize * 1024 * 1024) as *mut u8);
    let mut c = cpu::current_cpu();
    unsafe { c.enable_interrupts(); }
        
    // we're going to now enter the scheduler to do the rest
    let bootstrapped_thunk = move || { 
        bootstrapped_main(magic, info); 
    };
    
    scheduler::get_scheduler().schedule(box bootstrapped_thunk);
    scheduler::get_scheduler().bootstrap_start(); // okay, scheduler, take it away!
    unreachable!();
}

fn bootstrapped_main(magic: u32, info: *mut multiboot_info) {
    unsafe {
        let mut c = cpu::current_cpu();
        unsafe { c.enable_interrupts(); }
        c.make_keyboard(put_char);
        
        debug!("kernel main thread start!");

        test_allocator();
        
        
        if magic != multiboot::MULTIBOOT_BOOTLOADER_MAGIC {
            panic!("Multiboot magic is invalid");
        } else {
            debug!("Multiboot magic is valid. Info at 0x{:x}", info as u32);
            (*info).multiboot_stuff();
        }
        
        
        debug!("Going to interrupt: ");
        cpu::current_cpu().test_interrupt();
        debug!("    back from interrupt!");
        
        pci_stuff();
        
        scheduler::thread_stuff();
        
        info!("Kernel main thread is done!");
  }
}

fn pci_stuff() {
  let mut pci = Pci::new();
  pci.init();
  let mut drivers = pci.get_drivers();
  debug!("Found drivers for {} pci devices", drivers.len());
  match drivers.pop() {
    Some(mut driver) => {
      driver.init();
      net::NetworkStack::new(driver).test().ok();
    }
    None => ()
  }

}

#[no_mangle]
pub extern "C" fn debug(s: &'static str, u: u32) {
  debug!("{} 0x{:x}", s, u)
}

#[no_mangle]
pub extern "C" fn __morestack() {
  unreachable!("__morestack");
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
  loop {}
}

#[no_mangle]
pub extern "C" fn callback() {
  debug!("    in an interrupt!");
}

// TODO(ryan): figure out what to do with these:

#[lang = "stack_exhausted"]
extern fn stack_exhausted() {}

#[lang = "eh_personality"]
extern fn eh_personality() {}

// for deriving
#[doc(hidden)]
mod std {
  pub use core::*;
}
