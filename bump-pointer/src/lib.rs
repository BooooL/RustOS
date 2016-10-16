#![feature(no_std)]
#![no_std]
#![allow(unstable)]

use core::prelude::*;

extern "C" {
  fn memmove(dest: *mut u8, src: *mut u8, count: u32);
}


static mut allocator: BumpPointer = BumpPointer {
  start: 0 as *mut u8,
  stop:  0 as *mut u8,
};

pub fn set_allocator(start: *mut u8, stop: *mut u8) {
  unsafe {
    allocator = BumpPointer::new(start, stop);
  }
}

pub trait Allocator
{
  fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8>;

  fn deallocate(&mut self, ptr: *mut u8, old_size: usize, align: usize);

  fn reallocate(&mut self, ptr: *mut u8, old_size: usize, size: usize,
                align: usize) -> Option<*mut u8>
  {
    let attempt = self.allocate(size, align);
    if let Some(new) = attempt {
      unsafe { memmove(new, ptr, old_size as u32) };
      self.deallocate(ptr, old_size, align);
    }
    attempt
  }

  fn reallocate_inplace(&mut self, _ptr: *mut u8, old_size: usize, _size: usize,
                        _align: usize) -> usize
  {
    old_size
  }

  fn usable_size(&mut self, size: usize, _align: usize) -> usize
  {
    size
  }
  //fn stats_print(&mut self);

  fn debug(&mut self) -> (*mut u8, usize);
}

pub struct BumpPointer {
  start: *mut u8,
  stop:  *mut u8,
}

impl BumpPointer
{
  pub fn new(start: *mut u8, stop: *mut u8) -> BumpPointer {
    return BumpPointer { start: start, stop: stop };
  }
}

impl Allocator for BumpPointer
{
  #[inline]
  fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8>
  {
    let aligned: usize = {
      let a = self.start as usize + align - 1;
      a - (a % align)
    };
    let new_start = aligned + size;

    if new_start > self.stop as usize {
      unreachable!("allocator done!");
      None
    } else {
      self.start = new_start as *mut u8;
      Some(aligned as *mut u8)
    }
  }

  #[inline]
  fn deallocate(&mut self, _ptr: *mut u8, _old_size: usize, _align: usize) { }

  #[inline]
  fn debug(&mut self) -> (*mut u8, usize) {
    (self.start, self.stop as usize - self.start as usize)
  }
}

pub unsafe fn allocate(size: usize, align: usize) -> *mut u8 {
  match allocator.allocate(size, align) {
      Some(ptr) => ptr,
      None      => 0 as *mut u8
  }
}

pub unsafe fn deallocate(ptr: *mut u8, old_size: usize, align: usize) {
  allocator.deallocate(ptr, old_size, align)
}

pub unsafe fn reallocate(ptr: *mut u8, old_size: usize, size: usize,
                              align: usize) -> *mut u8 {
  match allocator.reallocate(ptr, old_size, size, align) {
    Some(ptr) => ptr,
    None      => 0 as *mut u8
  }
}

pub unsafe fn reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize,
                                      align: usize) -> usize {
  allocator.reallocate_inplace(ptr, old_size, size, align)
}

pub fn usable_size(size: usize, align: usize) -> usize {
  unsafe {
    allocator.usable_size(size, align)
  }
}

pub fn stats_print() {}
