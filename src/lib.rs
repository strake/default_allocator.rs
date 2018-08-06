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

extern "Rust" {
    #[cfg_attr(not(feature = "stable-rust"), rustc_allocator_nounwind)]
    fn __rust_alloc(size: usize, align: usize) -> *mut u8;
    #[cfg_attr(not(feature = "stable-rust"), rustc_allocator_nounwind)]
    fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize);
    #[cfg_attr(not(feature = "stable-rust"), rustc_allocator_nounwind)]
    fn __rust_realloc(ptr: *mut u8,
                      old_size: usize,
                      align: usize,
                      new_size: usize) -> *mut u8;
    #[cfg_attr(not(feature = "stable-rust"), rustc_allocator_nounwind)]
    fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8;
    #[cfg_attr(not(feature = "stable-rust"), rustc_allocator_nounwind)]
    fn __rust_grow_in_place(ptr: *mut u8,
                            old_size: usize,
                            old_align: usize,
                            new_size: usize) -> u8;
    #[cfg_attr(not(feature = "stable-rust"), rustc_allocator_nounwind)]
    fn __rust_shrink_in_place(ptr: *mut u8,
                              old_size: usize,
                              old_align: usize,
                              new_size: usize) -> u8;
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Heap;

unsafe impl Alloc for Heap {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(__rust_alloc(layout.size(), layout.align().get()))
            .ok_or(AllocErr::Exhausted { request: layout })
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        __rust_dealloc(ptr.as_ptr(), layout.size(), layout.align().get())
    }

    #[inline]
    unsafe fn realloc(&mut self, ptr: NonNull<u8>, layout: Layout, new_size: usize)
                      -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(__rust_realloc(ptr.as_ptr(), layout.size(), layout.align().get(), new_size))
            .ok_or(AllocErr::Exhausted { request: layout })
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        NonNull::new(__rust_alloc_zeroed(layout.size(), layout.align().get()))
            .ok_or(AllocErr::Exhausted { request: layout })
    }

    #[inline]
    unsafe fn resize_in_place(&mut self, ptr: NonNull<u8>, layout: Layout, new_size: usize)
                              -> Result<(), CannotReallocInPlace> {
        use ::core::cmp::Ord;
        use ::core::cmp::Ordering::*;
        if 0 == match Ord::cmp(&new_size, &layout.size()) {
            Greater => __rust_grow_in_place(ptr.as_ptr(), layout.size(), layout.align().get(), new_size),
            Less  => __rust_shrink_in_place(ptr.as_ptr(), layout.size(), layout.align().get(), new_size),
            Equal => 1,
        } { Err(CannotReallocInPlace) } else { Ok(()) }
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
