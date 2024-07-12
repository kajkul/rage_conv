use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LibcFreer<T: ?Sized + Send + Sync> {
    ptr: *mut c_void,
    value: ManuallyDrop<T>,
}

impl<T: Send + Sync> LibcFreer<T> {
    pub const unsafe fn new(value: T, ptr: *mut c_void) -> LibcFreer<T> {
        LibcFreer {
            ptr,
            value: ManuallyDrop::new(value),
        }
    }
}

impl<T: ?Sized + Send + Sync> Deref for LibcFreer<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: ?Sized + Send + Sync> DerefMut for LibcFreer<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: ?Sized + Send + Sync> Drop for LibcFreer<T> {
    fn drop(&mut self) {
        println!("freeing pointer: {:?}", self.ptr);
        unsafe { libc::free(self.ptr) }
    }
}

unsafe impl<T: ?Sized + Send + Sync> Send for LibcFreer<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for LibcFreer<T> {}
