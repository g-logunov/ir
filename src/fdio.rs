/// Slightly higher-level file descriptor IO operations.

extern crate libc;

use crate::err::{Error, Result};
use crate::sys;
use crate::sys::fd_t;
use std::string::String;

//------------------------------------------------------------------------------

pub fn read(fd: fd_t, buf: &mut Vec<u8>, len: usize) -> Result<()> {
    match sys::read(fd, buf, len)? {
        n if n as usize == len => Ok(()),
        0 => Err(Error::Eof),
        // FIXME: Handle short read.
        n => panic!("short read: {} {}", len, n),
    }
}

/// Reads a string from a file descriptor.  See `write_str`.
pub fn read_str(fd: fd_t) -> Result<String> {
    let len = read_usize(fd)?;
    let mut buf = Vec::with_capacity(len);
    read(fd, &mut buf, len)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn read_usize(fd: fd_t) -> Result<usize> {
    let mut data: [u8; 8] = [0; 8];
    match unsafe {
        libc::read(fd, &mut data as *mut [u8] as *mut libc::c_void, 8)
    }
    {
        -1 => Err(Error::last_os_error()),
        0 => Err(Error::Eof),
        8 => Ok(usize::from_ne_bytes(data)),
        ret => panic!("read_usize: read returned {}", ret),
    }
}

pub fn write(fd: fd_t, data: &[u8]) -> Result<()> {
    match sys::write(fd, data)? {
        n if n as usize == data.len() => Ok(()),
        0 => Err(Error::Eof),
        // FIXME: Handle short write.
        n => panic!("short write: {} {}", data.len(), n),
    }
}

/// Writes a string to `fd`.
///
/// First writes the string length as NE usize, followed by the UTF-8 bytes of
/// the string.  Use `read_str` to read.
pub fn write_str(fd: fd_t, s: &str) -> Result<()> {
    let bytes = s.as_bytes();
    write_usize(fd, bytes.len())?;
    write(fd, bytes)
}

pub fn write_usize(fd: fd_t, val: usize) -> Result<()> {
    write(fd, &val.to_ne_bytes())
}

