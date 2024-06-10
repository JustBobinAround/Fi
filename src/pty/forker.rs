use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use libc::{
    cfmakeraw, ioctl, tcgetattr, tcsetattr, termios as Termios, winsize, STDOUT_FILENO, TCSANOW,
    TIOCGWINSZ,
};
lazy_static::lazy_static! {
    static ref PRIOR_TERM_MODE: Arc<Mutex<Option<Termios>>> = Arc::new(Mutex::new(None));
}

use std::{
    mem,
    sync::{Arc, Mutex},
    fmt::Debug, 
    fs::{self, File}, 
    io::{self, Read, Write}, 
    os::unix::{
        io::{IntoRawFd, RawFd},
        prelude::AsRawFd,
    },
};
fn get_terminal_attr(fd: RawFd) -> Option<Termios> {
    unsafe {
        let mut termios = mem::zeroed();
        if tcgetattr(fd, &mut termios)==-1 {
            None
        } else {
            Some(termios)
        }
    }
}
pub(crate) fn disable_raw_mode() -> io::Result<()> {
    if let Ok(mut original_mode) = PRIOR_TERM_MODE.lock() {
        if let Some(original_mode_ios) = original_mode.as_ref() {
            let tty = tty_fd()?;
            unsafe { tcsetattr(tty.raw_fd(), TCSANOW, original_mode_ios) };
            // Keep it last - remove the original mode only if we were able to switch back
            *original_mode = None;
        }
    }


    Ok(())
}
pub(crate) fn enable_raw_mode() -> io::Result<()> {
    if let Ok(mut prior_mode) = PRIOR_TERM_MODE.lock() {
        if prior_mode.is_some() {
            return Ok(());
        }

        let tty = tty_fd().unwrap();
        let fd = tty.raw_fd();
        let mut ios = get_terminal_attr(fd).unwrap();
        let original_mode_ios = ios;
        unsafe { cfmakeraw(&mut ios) };
        unsafe { tcsetattr(fd, TCSANOW, &ios) };

        // Keep it last - set the original mode only if we were able to switch to the raw mode
        *prior_mode = Some(original_mode_ios);
    }

    Ok(())
}

use libc::size_t;
/// A file descriptor wrapper.
#[derive(Debug)]
pub struct FileDesc {
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

    /// Returns the underlying file descriptor.
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

pub fn tty_fd() -> io::Result<FileDesc> {
    let (fd, close_on_drop) = if unsafe { libc::isatty(libc::STDIN_FILENO) == 1 } {
        (libc::STDIN_FILENO, false)
    } else {
        (
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/tty")
                .unwrap()
                .into_raw_fd(),
            true,
        )
    };

    Ok(FileDesc::new(fd, close_on_drop))
}

pub fn old_main() -> io::Result<()> {
    let mut error_log = String::new();
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    stdout.write(crate::ascii::escapes::Escape::EnterAltScreen.to_string().as_bytes());
    stdout.flush();
    enable_raw_mode();
    //terminal::enable_raw_mode()?;
    let mut buffer = [0; 1];
    let pty_system = native_pty_system();

    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    }).expect("oOF");

    let mut cmd = CommandBuilder::new("bash");
    cmd.arg("-l");

    let mut child = pair.slave.spawn_command(cmd).expect("oOF");

    let reader = Arc::new(Mutex::new(pair.master.try_clone_reader().expect("OOF")));
    let writer = Arc::new(Mutex::new(pair.master.take_writer().expect("OOF")));
    let escaped = Arc::new(Mutex::new(true));

    let handler = std::thread::spawn(move || {
        let mut s: [u8; 10000] = [0;10000];
        let mut stdout = io::stdout();
        let mut last_x = 0;
        let mut last_y = 0;

        let mut was_escaped = false;
        while let Ok(mut reader) = reader.lock() {
            let n = reader.read(&mut s).unwrap();

            //stdout.queue(cursor::MoveTo(last_x,last_y));
            let _ = stdout.write(&s[0..n]);
            stdout.flush();
        };
        return;
    });

    //stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.write(crate::ascii::escapes::Escape::ClearAll.to_string().as_bytes());
    stdout.flush();

    for y in 0..40 {
        for x in 0..150 {
            if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
                // in this loop we are more efficient by not flushing the buffer.
                //stdout
                    //.queue(cursor::MoveTo(x,y))?
                    //.queue(style::PrintStyledContent("█".magenta()))?;
            }
        }
    }


    loop {
        if escaped.lock().is_ok_and(|b| *b==true) {
            if buffer[0] == 29 {
                if let Ok(mut escaped) = escaped.lock() {
                    *escaped = true;
                };
            }
            stdout.flush()?;
//            stdout
//                .queue(cursor::MoveTo(0,23))?
//                .queue(style::Print("NORMAL"))?
//                .queue(cursor::MoveTo(0,0))?;
            match buffer[0] as char {
                'q' => {break;},
                '\n' => {},
                'i' => {
                    if let Ok(mut escaped) = escaped.lock() {
                        *escaped = false;
                    };
                },


                _ => { 
//                    stdout
//                        .queue(cursor::MoveTo(70,23))?
//                        .queue(style::Print(&format!("   ")))?
//                        .queue(cursor::MoveTo(70,23))?
//                        .queue(style::Print(&format!("{}", buffer[0])))?
//                        .queue(cursor::MoveTo(0,0))?;
                }
            }        
            stdout.flush()?;
            stdin.read(&mut buffer)?;
        } else {
//            stdout
//                .queue(cursor::MoveTo(0,23))?
//                .queue(style::Print("-- INSERT --"))?
//                .queue(cursor::MoveTo(0,0))?;
            while child.try_wait().is_ok_and(|r| r.is_none())  {
                stdout.flush()?;
                if buffer[0] == 29 {
                    if let Ok(mut escaped) = escaped.lock() {
                        *escaped = true;
                        break;
                    };
                } else {
                    if let Ok(mut writer) = writer.lock() {
                        writer.write(&buffer);
                        //write!(writer, "{}", buffer[0]).expect("oOF");
                    };
                }
                stdout.flush()?;
                stdin.read(&mut buffer)?;
            }
        }
    }

    disable_raw_mode();
    stdout.write(crate::ascii::escapes::Escape::ExitAltScreen.to_string().as_bytes());
//    terminal::disable_raw_mode()?;
//    stdout.execute(terminal::LeaveAlternateScreen)?;
    stdout.flush()?;
    return Ok(());
}

