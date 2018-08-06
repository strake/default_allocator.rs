// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![no_std]

#![cfg_attr(not(feature = "stable-rust"), feature(rustc_attrs))]

extern crate loca;

use core::{ptr::NonNull, usize};

pub use loca::*;

#[derive(Copy, Clone, Default, Debug)]
pub struct Heap;

#[link(name = "c")]
extern "C" {
    fn aligned_alloc(align: usize, size: usize) -> *mut u8;
    fn realloc(_: *mut u8, _: usize) -> *mut u8;
    fn free(_: *mut u8);
}

unsafe impl Alloc for Heap {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(aligned_alloc(layout.align().get(), layout.size()))
            .ok_or(AllocErr::Exhausted { request: layout })
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, _layout: Layout) {
        free(ptr.as_ptr())
    }

    #[inline]
    unsafe fn realloc(&mut self, ptr: NonNull<u8>, layout: Layout, new_size: usize)
                      -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(realloc(ptr.as_ptr(), new_size))
            .ok_or(AllocErr::Exhausted { request: layout })
    }
}

#[cfg(test)]
mod tests {
    use ::{Heap, Alloc, Layout};

    #[test]
    fn allocate_zeroed() {
        unsafe {
            let layout = Layout::from_size_align(1024, 1).unwrap();
            let ptr = Heap.alloc_zeroed(layout.clone()).unwrap();

            let end = ptr.offset(layout.size() as isize);
            let mut i = ptr;
            while i < end {
                assert_eq!(*i, 0);
                i = i.offset(1);
            }
            Heap.dealloc(ptr, layout);
        }
    }
}
