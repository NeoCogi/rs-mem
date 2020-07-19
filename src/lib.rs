//
// Copyright 2020-Present (c) Raja Lehtihet & Wael El Oraiby
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software without
// specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//
#![no_std]
#![allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals)]
#[allow(improper_ctypes)]

pub unsafe fn allocRaw(size: usize) -> *mut u8 {
    //let addr = libc::memalign(core::mem::size_of::<usize>(), size) as *mut u8;
    let addr = libc::calloc(core::mem::size_of::<usize>(), size) as *mut u8;
    //libc::memset(addr as *mut libc::c_void, 0, size);
    addr
}

pub unsafe fn freeRaw(arr: *mut u8) {
    libc::free(arr as *mut libc::c_void);
}

pub unsafe fn alloc<T>() -> *mut T {
    allocRaw(core::mem::size_of::<T>()) as *mut T
}

pub unsafe fn free<T>(t: *mut T) {
    freeRaw(t as *mut u8)
}

// TODO: change this to const generics when they become stable and return a slice
pub unsafe fn allocArray<T>(count: usize) -> *mut T {
    allocRaw(core::mem::size_of::<T>() * count) as *mut T
}

// TODO: change this to slice once const generics stable
pub unsafe fn freeArray<T>(ptr: *mut T, count: usize) {
    let arr      = core::slice::from_raw_parts_mut(ptr, count); // this will keep a pointer (will not free it)
    for i in 0..count {
        ::core::ptr::drop_in_place(&arr[i] as *const T as *mut T);
    }
    free(ptr);
}

#[repr(C)]
pub struct Unique<T: ?Sized> {
    ptr         : *mut T,
    _marker     : ::core::marker::PhantomData<T>,
}

impl<T> Unique<T> {
    pub fn new(ptr: *mut T) -> Self { Self { ptr : ptr, _marker: ::core::marker::PhantomData } }
    pub fn getMutPtr(&mut self) -> *mut T { self.ptr }
    pub fn getPtr(&self) -> *const T { self.ptr }
}

#[repr(C)]
pub struct Box<T>{
    uptr: Unique<T>
}

impl<T> Box<T> {
    /// Allocates memory on the heap and then places `x` into it.
    ///
    /// # Examples
    ///
    /// ```
    /// let five = Box::new(5);
    /// ```
    #[inline(always)]
    pub fn new(x: T) -> Box<T> {
        unsafe {
            let addr = alloc::<T>();
            *addr = x;
            Self { uptr: Unique::new(addr) }
        }
    }

    pub fn asRef(&self) -> &T { unsafe { &(*self.uptr.getPtr()) } }
    pub fn asMut(&mut self) -> &T { unsafe { &mut (*self.uptr.getMutPtr()) } }
    pub fn intoRaw(self) -> *mut T {
        let m = ::core::mem::ManuallyDrop::new(self);
        m.uptr.ptr
    }

    pub fn fromRaw(raw: *mut T) -> Self {
        Self { uptr: Unique::new(raw) }
    }

    pub fn unbox(self) -> T {
        unsafe {
            let ptr = self.uptr.ptr;
            let v = self.intoRaw().read();
            free(ptr);
            v
        }
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            let addr = self.uptr.getMutPtr();
            ::core::ptr::drop_in_place(addr);
            free(addr);
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    #[test]
    fn testDrop() {
        let _b0 = Box::new(1234);
        let _b1 = Box::new(1234345);
        let mut v = std::vec::Vec::new();
        for i in 0..100 {
            v.push(i);
        }
        let _bv = Box::new(v);
    }

    #[test]
    fn testDropVecVec() {
        let _b0 = Box::new(1234);
        let _b1 = Box::new(1234345);
        let mut v = std::vec::Vec::new();
        for _ in 0..100 {
            let mut vj = std::vec::Vec::new();
            for j in 0..100 {
                vj.push(j);
            }
            v.push(vj);
        }
        let _bv = Box::new(v);
    }

    #[test]
    fn testBoxUnbox() {
        let b = Box::new(1234);
        let _v = b.unbox();
    }

    #[test]
    fn testBoxUnboxVecVec() {
        let _b0 = Box::new(1234);
        let _b1 = Box::new(1234345);
        let mut v = std::vec::Vec::new();
        for _ in 0..100 {
            let mut vj = std::vec::Vec::new();
            for j in 0..100 {
                vj.push(j);
            }
            v.push(vj);
        }
        let v2 = Box::new(v);
        let _v3 = v2.unbox();
    }

    #[test]
    fn testBoxFromToRaw() {
        let b = Box::new(1234);
        let r = b.intoRaw();
        let _b = Box::fromRaw(r);
    }
}
