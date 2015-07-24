// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
//! Semaphores are very useful for concurrency.
//! Someone please write some documentation, kthx.

#[cfg(windows)]
mod platform {
    extern crate winapi;
    extern crate kernel32;
    use self::winapi::*;
    use self::kernel32::*;
    use std::io::{Error, Result};
    use std::ptr;
    pub struct Semaphore(HANDLE);
    impl Semaphore {
        pub fn new(initial: u32) -> Result<Semaphore> {
            let sem = unsafe {
                CreateSemaphoreW(ptr::null_mut(), initial as LONG, 0x10000000, ptr::null())
            };
            if sem.is_null() { Err(Error::last_os_error()) }
            else { Ok(Semaphore(sem)) }
        }
        pub fn release(&mut self) -> Result<()> {
            // Windows could also get the current count
            // sem_post doesn't return the current count, but is there another posix method that can?
            let err = unsafe { ReleaseSemaphore(self.0, 1, ptr::null_mut()) };
            if err == 0 { Err(Error::last_os_error()) }
            else { Ok(()) }
        }
        pub fn release_many(&mut self, count: u32) -> Result<()> {
            // See comment on release
            // Also does posix have a convenient way to release multiple?
            let err = unsafe {
                ReleaseSemaphore(self.0, count as LONG, ptr::null_mut())
            };
            if err == 0 { Err(Error::last_os_error()) }
            else { Ok(()) }
        }
        pub fn acquire(&mut self) -> Result<()> {
            let err = unsafe { WaitForSingleObject(self.0, INFINITE) };
            if err == 0 { Err(Error::last_os_error()) }
            else { Ok(()) }
        }
        /// Returns Ok(None) if it times out
        pub fn acquire_timeout_ms(&mut self, timeout: u32) -> Result<Option<()>> {
            let err = unsafe { WaitForSingleObject(self.0, timeout) };
            if err == 0 { Err(Error::last_os_error()) }
            else if err == WAIT_TIMEOUT { Ok(None) }
            else { Ok(Some(())) }
        }
    }
    impl Clone for Semaphore {
        fn clone(&self) -> Semaphore {
            let mut handle = ptr::null_mut();
            let pro = unsafe { GetCurrentProcess() };
            let err = unsafe {
                DuplicateHandle(pro, self.0, pro, &mut handle, 0, FALSE, DUPLICATE_SAME_ACCESS)
            };
            if err == 0 {
                let error = Error::last_os_error();
                panic!("Failed to clone semaphore! {:?}", error);
            }
            Semaphore(handle)
        }
    }
    impl Drop for Semaphore {
        fn drop(&mut self) {
            assert!(unsafe { CloseHandle(self.0) } != 0, "Failed to close semaphore handle")
        }
    }
    unsafe impl Send for Semaphore {}
}
pub use platform::Semaphore;
