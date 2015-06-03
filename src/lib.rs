#![no_std]


#![allow(improper_ctypes)]

#![feature(asm)]
#![feature(lang_items)]
#![feature(box_syntax)]

#![feature(core, alloc, collections)]
#![feature(no_std)]

// not directly used, but needed to link to llvm emitted calls
extern crate rlibc;

#[macro_use]
//extern crate std; // for useful macros and IO interfaces
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
//use thread::scheduler;

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

lazy_static! {
  static ref TEST: Vec<&'static str> = {
    let mut v = Vec::new();
    v.push("hi from lazy static");
    v
  };
}

#[no_mangle]
pub extern "C" fn main(magic: u32, info: *mut multiboot_info) {
    unsafe {
        bump_ptr::set_allocator((15usize * 1024 * 1024) as *mut u8, (20usize * 1024 * 1024) as *mut u8);
        terminal::init_global();
    debug!("kernel start!");
    
    panic::init();
        
    test_allocator();
    
    if magic != multiboot::MULTIBOOT_BOOTLOADER_MAGIC {
      panic!("Multiboot magic is invalid");
    } else {
      debug!("Multiboot magic is valid. Info at 0x{:x}", info as u32);
      (*info).multiboot_stuff();
    }

    debug!("{}", (**TEST)[0]);

    let mut c = cpu::current_cpu();
    c.make_keyboard(put_char);

    c.enable_interrupts();
    debug!("Going to interrupt: ");
    c.test_interrupt();
    debug!("    back from interrupt!");

    //debug!("start scheduling...");

    //scheduler::thread_stuff();

    pci_stuff();

    info!("Kernel is done!");
    loop {
      c.idle()
    }
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
  loop { } //TODO(ryan) should I do anything here?
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
  unsafe { core::intrinsics::abort(); }
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
