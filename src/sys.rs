extern crate libc;

use libc::{c_int, pid_t, rusage};
use std::ffi::CString;
use std::io;
use std::path::{Path, PathBuf};
use std::mem::MaybeUninit;
use std::string::String;
use std::vec::Vec;

use crate::environ::Env;

#[allow(non_camel_case_types)]
pub type fd_t = c_int;

//------------------------------------------------------------------------------

/// C-style char* array, containing a NULL-terminated array of pointers to
/// nul-terminated strings.
struct CStringVec {
    // Nul-terminated strings.
    // FIXME: We need to keep this around as it stores the actual strings
    // pointed to by `ptrs`, but Rust doesn't know this.  Should figure out how
    // to tell it.
    #[allow(dead_code)]
    strs: Vec<CString>,

    // NULL-terminated vector of char* pointers.
    ptrs: Vec<*const i8>,
}

impl CStringVec {
    pub fn as_ptr(&self) -> *const *const i8 { self.ptrs.as_ptr() as *const *const i8 }
}

impl<T> From<T> for CStringVec
where T: IntoIterator<Item = String>
{
    fn from(strings: T) -> Self {
        // Build nul-terminated strings.
        let strs
            = strings.into_iter()
            .map(|s| { CString::new(s).unwrap() })
            .collect::<Vec<_>>();

        // Grab their pointers into an array.
        let mut ptrs
            = strs.iter()
            .map(|s| {
                s.as_ptr() as *const i8
            })
            .collect::<Vec<_>>();
        // NULL-terminate the pointer array.
        ptrs.push(std::ptr::null());

        Self { strs, ptrs }
    }
}

//------------------------------------------------------------------------------

pub fn close(fd: fd_t) -> io::Result<()> {
    let res = unsafe { libc::close(fd) };
    match res {
        -1 => Err(io::Error::last_os_error()),
         0 => Ok(()),
         _ => panic!("close returned {}", res),
    }
}

pub fn dup2(fd: fd_t, fd2: fd_t) -> io::Result<()> {
    let res = unsafe { libc::dup2(fd, fd2) };
    match res {
        -1 => Err(io::Error::last_os_error()),
        _ if res == fd2 => Ok(()),
        _ => panic!("dup2 returned {}", res),
    }
}

pub fn execv(exe: String, args: Vec<String>) -> io::Result<()> {
    let res = unsafe {
        libc::execv(
            exe.as_ptr() as *const i8,
            CStringVec::from(args).as_ptr())
    };
    // execv only returns on failure, with result -1.
    assert!(res == -1);
    Err(io::Error::last_os_error())
}

pub fn execve(exe: String, args: Vec<String>, env: Env) -> io::Result<()> {
    // Construct NAME=val strings for env vars.
    let env: Vec<String> = env.into_iter().map(|(n, v)| {
        format!("{}={}", n, v)
    }).collect();

    let res = unsafe {
        libc::execve(
            CString::new(exe).unwrap().as_ptr() as *const i8,
            CStringVec::from(args).as_ptr(), 
            CStringVec::from(env).as_ptr())
    };
    // execve only returns on failure, with result -1.
    assert!(res == -1);
    Err(io::Error::last_os_error())
}

pub fn fork() -> io::Result<pid_t> {
    let child_pid = unsafe { libc::fork() };
    assert!(child_pid >= -1);
    match child_pid {
        -1 => Err(io::Error::last_os_error()),
        _ if child_pid >= 0 => Ok(child_pid),
        _ => panic!("fork returned {}", child_pid),
    }
}

pub fn getpid() -> pid_t {
    unsafe { libc::getpid() }
}

pub fn mkstemp(template: String) -> io::Result<(PathBuf, fd_t)> {
    let path = CString::new(template)?;
    let (fd, path) = unsafe {
        let ptr = path.into_raw();
        (libc::mkstemp(ptr), CString::from_raw(ptr))
    };
    match fd {
        -1 => Err(io::Error::last_os_error()),
        _ if fd >= 0 => Ok((PathBuf::from(path.into_string().unwrap()), fd)),
        _ => panic!("mkstemp returned {}", fd),
    }
}

pub fn open(path: &Path, oflag: c_int, mode: c_int) -> io::Result<fd_t> {
    let fd = unsafe {
        libc::open(
            CString::new(path.to_str().unwrap()).unwrap().as_ptr() as *const i8,
            oflag, mode)
    };
    match fd {
        -1 => Err(io::Error::last_os_error()),
        _ if fd >= 0 => Ok(fd),
        _ => panic!("open returned {}", fd)
    }
}

pub fn wait4(pid: pid_t, options: c_int) -> io::Result<(pid_t, c_int, rusage)> {
    let mut status: c_int = 0;
    let mut usage = MaybeUninit::<rusage>::uninit();
    let res = unsafe { 
        libc::wait4(pid, &mut status, options, usage.as_mut_ptr())
    };
    match res {
        -1 => Err(io::Error::last_os_error()),
        child_pid => Ok((child_pid, status, unsafe { usage.assume_init() })),
    }
}

