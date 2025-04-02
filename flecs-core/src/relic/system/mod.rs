pub mod info;

pub use super::{Error, Result};
use libc::c_char;
use std::ffi::CStr;
use std::mem::MaybeUninit;

pub fn hostname() -> std::io::Result<String> {
    const BUFFER_LEN: usize = 256;
    let mut buffer: MaybeUninit<[c_char; BUFFER_LEN]> = MaybeUninit::uninit();
    if unsafe { libc::gethostname(buffer.as_mut_ptr() as *mut c_char, BUFFER_LEN) } != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        let hostname = unsafe {
            let buffer = buffer.assume_init();
            CStr::from_ptr(buffer.as_ptr())
        };
        Ok(hostname.to_string_lossy().to_string())
    }
}
