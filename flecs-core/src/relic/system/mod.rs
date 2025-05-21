pub mod info;

pub use super::{Error, Result};
use libc::c_char;
use std::ffi::CStr;
use std::mem::MaybeUninit;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum ReadHostnameError {
    #[error("Call to 'gethostname' failed: {0}")]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    CStr(#[from] std::ffi::FromBytesUntilNulError),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

pub fn hostname() -> Result<String, ReadHostnameError> {
    const BUFFER_LEN: usize = 256;
    let mut buffer: MaybeUninit<[u8; BUFFER_LEN]> = MaybeUninit::uninit();
    if unsafe { libc::gethostname(buffer.as_mut_ptr() as *mut c_char, BUFFER_LEN) } != 0 {
        Err(ReadHostnameError::IO(std::io::Error::last_os_error()))
    } else {
        let hostname = unsafe { buffer.assume_init() };
        let hostname = CStr::from_bytes_until_nul(&hostname)?;
        let hostname = hostname.to_str()?;
        Ok(hostname.to_string())
    }
}
