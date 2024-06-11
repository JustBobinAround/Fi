use libc::{
    cfmakeraw, tcgetattr, tcsetattr, termios as Termios, TCSANOW,
};

use std::{
    fmt::Debug, fs, io::{self, Error}, mem, os::unix::{
        io::{IntoRawFd, RawFd},
        prelude::AsRawFd,
    }, 
    sync::{Arc, Mutex} 
};

fn get_term_attr(fd: RawFd) -> io::Result<Termios>{
    unsafe {
        let mut termios = mem::zeroed();
        if tcgetattr(fd, &mut termios)==-1 {
            Err(Error::new(io::ErrorKind::Other, "Could not get terminal attr"))
        } else {
            Ok(termios)
        }
    }
}

fn no_raw(mut prior_mode: Option<Termios>) -> io::Result<Option<Termios>> {
    if let Some(new_prior_mode) = prior_mode.as_ref() {
        let tty = tty_fd()?;
        unsafe { tcsetattr(tty.raw_fd(), TCSANOW, new_prior_mode) };
        prior_mode = None;
    }
    Ok(prior_mode)
}

fn set_raw(mut prior_mode: Option<Termios>) -> io::Result<Option<Termios>> {
    if prior_mode.is_some(){
        return Ok(prior_mode)
    }
    let tty = tty_fd()?;
    let fd = tty.raw_fd();
    let mut ios = get_term_attr(fd)?;
    let new_prior_mode = ios;

    unsafe { 
        cfmakeraw(&mut ios); 
        tcsetattr(fd, TCSANOW, &ios);
    };

    prior_mode = Some(new_prior_mode);

    Ok(prior_mode)
}

pub fn raw_mode(prior_mode: Option<Termios>) -> io::Result<Option<Termios>> {
    if prior_mode.is_none(){
        set_raw(prior_mode)
    } else {
        no_raw(prior_mode)
    }
}

use libc::size_t;
#[derive(Debug)]
struct FileDesc {
    fd: RawFd,
    close_on_drop: bool,
}

impl FileDesc {
    pub fn new(fd: RawFd, close_on_drop: bool) -> FileDesc {
        FileDesc { fd, close_on_drop }
    }

    pub fn read(&self, buffer: &mut [u8], size: usize) -> io::Result<usize> {
        let result = unsafe {
            libc::read(
                self.fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                size as size_t,
            )
        };

        if result < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(result as usize)
        }
    }

    pub fn raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        if self.close_on_drop {
            let _ = unsafe { libc::close(self.fd) };
        }
    }
}

impl AsRawFd for FileDesc {
    fn as_raw_fd(&self) -> RawFd {
        self.raw_fd()
    }
}

fn tty_fd() -> io::Result<FileDesc> {
    let is_tty = unsafe{libc::isatty(libc::STDIN_FILENO) == 1};
    if is_tty {
        Ok(FileDesc::new(libc::STDIN_FILENO, false))
    } else {
        Ok(FileDesc::new(
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/tty")?
                .into_raw_fd(),
            true))
    }
}
