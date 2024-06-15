use libc::termios as Termios;
use std::ffi::CString;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use libc::{pid_t, TIOCGSID, ioctl, readlink};
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::sync::{Arc, Mutex};
use std::io::{self, Error, Read, Stdout, Write};
use super::raw_mode::raw_mode;
use crate::ascii::escapes::{Escape, Sequence, ParsableSequence};
use crate::logger::log_message;

pub struct PTerminal{
    writer: Stdout,
    to_write: Vec<u8>,
    buffer_hist: Vec<Sequence>,
    raw_mode: Option<Termios>,
    pub join_handler: bool,
    size_x: u16,
    size_y: u16,
    offset_x: u32,
    offset_y: u32,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    pty_pair: PtyPair,
    pty_writer: Box<dyn Write + Send>,
    pty_reader: Arc<Mutex<Box<dyn Read + Send>>>,
}

impl PTerminal {
    pub fn new(
        size_x: u16,
        size_y: u16,
        offset_x: u32,
        offset_y: u32,
    ) -> io::Result<(JoinHandle<()>,Arc<Mutex<PTerminal>>)> {
        let mut cmd = CommandBuilder::new("bash");
        cmd.arg("-l");
        let raw_mode = raw_mode(None)?;

        let pair = native_pty_system().openpty(PtySize {
            rows: size_y,
            cols: size_x,
            pixel_width: 0,
            pixel_height: 0,
        });

        let pair = match pair {
            Ok(pair) => {pair},
            Err(_) => {return Err(Error::new(io::ErrorKind::Other, "failed to make pty pair"))}
        };


        let child = match pair.slave.spawn_command(cmd) {
            Ok(child) => {child},
            Err(_) => {return Err(Error::new(io::ErrorKind::Other, "failed to spawn process"))}
        };


        let reader = Arc::new(Mutex::new(pair.master.try_clone_reader().expect("OOF")));
        let writer = pair.master.take_writer().expect("OOF");
        let to_write = Vec::new();
        let buffer_hist = Vec::new();
        let join_handler = Arc::new(Mutex::new(false));

        let p_term = Arc::new(Mutex::new(PTerminal { 
            writer: io::stdout(),
            to_write,
            buffer_hist,
            raw_mode,
            join_handler: false,
            size_x,
            size_y,
            offset_x,
            offset_y,
            child,
            pty_pair: pair,
            pty_writer: writer,
            pty_reader: reader,
        }));
        let p_term_2 = p_term.clone();
        let p_term_3 = p_term.clone();
        if let Ok(mut p_term) = p_term.lock() {
            p_term.queue(Sequence::Escape(vec![
                Escape::EnterAltScreen,
                Escape::ClearAll
            ]))?;

            p_term.flush()?;
        }

        let pty_handler = std::thread::spawn(move || {
            let mut count = 0;
            let mut s: [u8; 1] = [0;1];
            loop{
            if let Ok(mut p_term) = p_term_2.lock() {
                log_message("a1");
                if p_term.join_handler {
                    break;
                }
                log_message("a2");
                let test = p_term.pty_reader.clone();
                drop(p_term);
                log_message("a3");
                if let Ok(mut test) = test.lock() {
                    log_message("special2");
                    let seqs = Sequence::parse_writer(&mut test);
                    log_message("special");
                    if let Ok(mut p_term) = p_term_2.lock() {
                        if p_term.join_handler {
                            break;
                        }
                        for seq in seqs {
                            match seq {
                                Sequence::Text(t) => {
                                    p_term.to_write.push(t as u8);
                                },
                                Sequence::Escape(escs) => {
                                    for esc in escs {
                                        p_term.to_write.append(&mut esc.to_string().into_bytes());
                                    }
                                }
                            }
                        }
                        log_message("a4");
                        p_term.flush();
                    }
                };
                log_message("a5");
            }
            }
            log_message("pty_handler exited");
            return;
        });

        let _key_listener_handler = std::thread::spawn(move || {
            let mut key_buffer = [0;1];
            let mut stdin = io::stdin();
            let mut escaped = true;
            loop {
            if let Ok(mut p_term) = p_term_3.try_lock() {
                if p_term.join_handler {
                    break;
                }
                log_message("b1");
                if escaped {
                log_message("ba2");
                    if key_buffer[0] == 29 {
                        escaped = true;
                    }
                log_message("ba3");
                    match key_buffer[0] as char {
                        'r' => {
                            if let Ok(pwd) = p_term.get_process_pwd() {
                                p_term.respawn(&pwd);
                            }                            
                        }
                        'q' => {
                            p_term.close();
                            log_message("closing all");
                            break;
                        },
                        '\n' => {},
                        'i' => {
                            escaped = false;
                        },
                        _ => {}
                    }        
                log_message("ba4");
                    drop(p_term);
                    let _ = stdin.read_exact(&mut key_buffer);
                log_message("ba5");
                } else {
                log_message("bb1");
                    if key_buffer[0] == 29 {
                        escaped = true;
                log_message("bb2");
                    } else {
                log_message("bb3");
                        (p_term.pty_writer).write(&key_buffer); 
                log_message("bb4");
                        p_term.pty_writer.flush();
                log_message("bb5");
                            //let _ = writer.write(&*key_buffer);
                    }
                    drop(p_term);
                log_message("bb6");
                    //let _ = stdin.read(&mut key_buffer);
                    let _ = stdin.read_exact(&mut key_buffer);
                log_message("bb7");
                }
            }}
            log_message("key log exited");
        });

        Ok((pty_handler,p_term))
    }


    pub fn get_process_pwd(&self) -> io::Result<String> {
        if let Some(process_id) = self.pty_pair.master.process_group_leader() {
        //if let Some(process_id) = child.process_id() {
            let mut target = vec![0u8; 4096];


            let path = format!("/proc/{}/cwd", process_id);

            match PTerminal::read_link_to_buf(&path, &mut target) {
                Ok(n) if n > 0 => return Ok(String::from_utf8_lossy(&target[..n]).to_string()),
                Ok(_) => {},
                Err(_) => {}
            }

            let mut sid: pid_t = 0;
            if unsafe { ioctl(process_id as i32, TIOCGSID, &mut sid) } != -1 {
                let path = format!("/proc/{}/cwd", sid);
                match PTerminal::read_link_to_buf(&path, &mut target) {
                    Ok(n) if n > 0 => return Ok(String::from_utf8_lossy(&target[..n]).to_string()),
                    Ok(_) => {},
                    Err(_) => {}
                }
            }

            return Err(io::Error::new(io::ErrorKind::NotFound, "process id returned None"));
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "process id returned None"));
        }
    }

    pub fn read_link_to_buf(path: &str, buf: &mut [u8]) -> io::Result<usize> {
        let c_path = CString::new(path)?;
        let n = unsafe { readlink(c_path.as_ptr(), buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        if n == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(n as usize)
        }
    }

    pub fn respawn(&mut self, working_dir: &String) -> io::Result<()>{
        let mut cmd = CommandBuilder::new("sh");
        cmd.arg("-c");
        cmd.arg(format!("cd {}; bash", working_dir));

        self.child.kill()?;
        self.child = match self.pty_pair.slave.spawn_command(cmd) {
            Ok(child) => {child},
            Err(_) => {
                return Err(Error::new(io::ErrorKind::Other, "failed to spawn process"))}
        };
        Ok(())
    }

    fn queue(&mut self, seq: Sequence) -> io::Result<()>{
        match seq {
            Sequence::Text(text) => {
                self.to_write.push(text as u8);
            },
            Sequence::Escape(escs) => {
                for esc in escs {
                    for b in esc.to_string().as_bytes().iter() {
                        self.to_write.push(*b);
                    }
                    //esc.send(&mut self.writer.lock())?;
                }
            }
        }

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()>{
        let mut writer = self.writer.lock();
        writer.write(&self.to_write)?;
        self.to_write.clear();

        writer.flush()?;

        Ok(())
    }

    pub fn close(&mut self) {
        self.child.kill();
        raw_mode(self.raw_mode);
        self.queue(Sequence::Escape(vec![Escape::ExitAltScreen]));
        self.flush();
        self.join_handler = true;
    }

}
