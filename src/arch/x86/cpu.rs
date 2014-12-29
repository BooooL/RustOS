use core::prelude::*;
use core::cell::UnsafeCell;

use alloc::boxed::Box;

//use io::{Reader, Writer};

use arch::idt::IDT;
use arch::gdt::GDT;

use arch::keyboard::Keyboard;

// TODO remove box hack. It says it has a global destructor but I don't know why
lazy_static_spin! {
  static CURRENT_CPU: *mut CPU = {
    unsafe {
      let cpu: Box<UnsafeCell<CPU>> = box UnsafeCell::new(CPU::new());
      let ret = cpu.get();
      ::core::mem::forget(cpu); // leak memory!
      ret
    }
  };
}

pub enum IRQ { // after remap
  Timer        = 0x20,
  PS2Keyboard  = 0x21,
  Cascade      = 0x22,
  COM2         = 0x23,
  COM1         = 0x24,
  LPT2         = 0x25,
  Floppy       = 0x26,
  LPT1         = 0x27,
  CmosClock    = 0x28,
  FreeOne      = 0x29,
  FreeTwo      = 0x2a,
  FreeThree    = 0x2b,
  PsMouse      = 0x2c,
  FPU          = 0x2d,
  PrimaryAta   = 0x2e,
  SecondaryAta = 0x2f
}

extern "C" {

  fn interrupt();

  fn debug(s: &str, u: u32);

}

#[allow(dead_code)]
pub struct CPU {
  gdt: GDT,
  idt: IDT,
  keyboard: Option<Keyboard>
  //ports: Ports
}

impl CPU {

  pub unsafe fn new() -> CPU {
    let mut gdt = GDT::new();

    gdt.identity_map();
    gdt.enable();

    PIC::master().remap_to(0x20);
    PIC::slave().remap_to(0x28);

    let mut idt = IDT::new();

    idt.enable();
    CPU { gdt: gdt, idt: idt, keyboard: None}
  }

  pub fn handle(&mut self, interrupt_number: u32) {
    match interrupt_number {
      0x20 => (), // timer
      0x21 => match &mut self.keyboard {
        &Some(ref mut k) => k.got_interrupted(),
        &None            => unsafe { debug("no keyboard installed", 0) }
      },
      _ => {debug!("interrupt with no handler: {:u}", interrupt_number); loop {};}
    }
    self.acknowledge_irq(interrupt_number);
  }

  pub unsafe fn register_irq(&mut self, irq: IRQ, handler: extern "C" fn () -> ()) {
    self.idt.add_entry(irq as u32, handler);
  }

  pub unsafe fn idle(&mut self) {
    asm!("hlt" ::::)
  }

  fn acknowledge_irq(&mut self, _: u32) {
    PIC::master().control_port.out_b(0x20); //TODO(ryan) ugly and only for master PIC
  }

  pub fn make_keyboard(&mut self, callback: fn (u8) -> ()) {
    self.keyboard = Some(Keyboard::new(callback, Port {port_number: 0x64}, Port {port_number: 0x60}));
    //self.register_irq(Keyboard, )
  }

  pub unsafe fn enable_interrupts(&mut self) {
    IDT::enable_interrupts();
  }

  pub fn disable_interrupts(&mut self) {
    IDT::disable_interrupts();
  }

  pub unsafe fn test_interrupt(&mut self) {
    interrupt();
  }

}

#[no_mangle]
pub extern "C" fn unified_handler(interrupt_number: u32) {
  unsafe { (**CURRENT_CPU).handle(interrupt_number); }
}

#[no_mangle]
pub extern "C" fn add_entry(idt: &mut IDT, index: u32, f: unsafe extern "C" fn() -> ()) {
  idt.add_entry(index, f);
}


struct PIC {
  control_port: Port,
  mask_port: Port,
  is_master: bool
}

impl PIC {

  fn master() -> PIC {
    PIC { control_port: Port::new(0x20), mask_port: Port::new(0x21), is_master: true}
  }

  fn slave() -> PIC {
    PIC { control_port: Port::new(0xA0), mask_port: Port::new(0xA1), is_master: false}
  }

  unsafe fn remap_to(&mut self, start: u8) {
    let icw1 = 0x11;
    let icw4 = 0x1;
    let enable_all = 0x00;
    let typ = if self.is_master { 0x2 } else { 0x4 };

    self.control_port.out_b(icw1);
    self.mask_port.write(&[start, typ, icw4, enable_all]).ok();
  }

}

pub struct Port {
  port_number: u16
}

impl Port {

  pub fn new(number: u16) -> Port {
    Port { port_number: number }
  }

  pub fn in_b(&mut self) -> u8 {
    let mut ret: u8;
    unsafe {
      asm!("inb $1, $0" : "={al}"(ret) :"{dx}"(self.port_number) ::)
    }
    return ret;
  }

  pub fn out_b(&mut self, byte: u8) {
    unsafe {
      asm!("outb $1, $0" :: "{dx}"(self.port_number), "{al}"(byte) ::)
    }
  }

  pub fn out_w(&mut self, word: u16) {
    unsafe {
      asm!("outw $1, $0" ::"{dx}"(self.port_number), "{ax}"(word) ::)
    }
  }

  pub fn in_w(&mut self) -> u16 {
    let mut ret: u16;
    unsafe {
      asm!("inw $1, $0" : "={ax}"(ret) :"{dx}"(self.port_number)::)
    }
    ret
  }

  pub fn out_l(&mut self, long: u32) {
    unsafe {
      asm!("outl $1, $0" ::"{dx}"(self.port_number), "{eax}"(long) ::)
    }
  }

  pub fn in_l(&mut self) -> u32 {
    let mut ret: u32;
    unsafe {
      asm!("inl $1, $0" : "={eax}"(ret) :"{dx}"(self.port_number)::)
    }
    ret
  }

  pub fn io_wait() {
    Port::new(0x80).out_b(0);
  }

}

impl Port
{
  fn read(&mut self, buf: &mut [u8]) -> Result<uint, ()> {
    for el in buf.iter_mut() {
      *el = self.in_b();
    }
    Ok(buf.len())
  }

  fn write(&mut self, buf: &[u8]) -> Result<(), ()> {
    for &byte in buf.iter() {
      self.out_b(byte);
    }
    Ok(())
  }
}

/*
impl Reader for Port {

  type Err = (); // TODO use bottom type

  //fn read_u8(&mut self) -> Result<u8, ()> {
  //  Ok(self.in_b())
  //}

  fn read(&mut self, buf: &mut [u8]) -> Result<uint, ()> {
    for el in buf.iter_mut() {
      *el = self.in_b();
    }
    Ok(buf.len())
  }

}

impl Writer for Port {

  type Err = ();

  //fn write_u8(&mut self, byte: u8) -> Result<(), ()> {
  //  self.out_b(byte);
  //  Ok(())
  //}

  fn write(&mut self, buf: &[u8]) -> Result<(), ()> {
    for &byte in buf.iter() {
      self.out_b(byte);
    }
    Ok(())
  }

}
 */
