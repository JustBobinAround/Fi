use libc::termios as Termios;
use std::ffi::CString;
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
    key_buffer: Arc<Mutex<[u8; 1]>>,
    size_x: u16,
    size_y: u16,
    offset_x: u32,
    offset_y: u32,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    pty_pair: PtyPair,
    pty_writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pty_reader: Arc<Mutex<Box<dyn Read + Send>>>,
}

impl PTerminal {
    pub fn new(
        size_x: u16,
        size_y: u16,
        offset_x: u32,
        offset_y: u32,
    ) -> io::Result<Arc<Mutex<PTerminal>>> {
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

        let child = Arc::new(Mutex::new(child));
        let child_2= child.clone();


        let reader = Arc::new(Mutex::new(pair.master.try_clone_reader().expect("OOF")));
        let reader_2 = reader.clone();
        let writer = Arc::new(Mutex::new(pair.master.take_writer().expect("OOF")));
        let writer_2 = writer.clone();
        let to_write = Vec::new();
        let buffer_hist = Vec::new();
        let key_buffer = Arc::new(Mutex::new([0;1]));
        let key_buffer_2 = key_buffer.clone();

        let p_term = Arc::new(Mutex::new(PTerminal { 
            writer: io::stdout(),
            to_write,
            buffer_hist,
            raw_mode,
            key_buffer,
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

        let _pty_handler = std::thread::spawn(move || {

            while let Ok(mut reader) = reader_2.lock() {
                if p_term_2.lock().is_ok_and(|j|j.join_handler) {
                    break;
                }
                if let Ok(mut p_term) = p_term_2.lock() {
                    let seqs = Sequence::parse_writer(&mut reader);
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
                    p_term.flush();
                }
            };
            log_message("pty_handler exited");
            return;
        });

        let _key_listener_handler = std::thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut escaped = true;
            while let Ok(mut key_buffer) = key_buffer_2.lock() {
                if p_term_3.lock().is_ok_and(|j|j.join_handler) {
                    break;
                }
                if escaped {
                    if key_buffer[0] == 29 {
                        escaped = true;
                    }
                    match key_buffer[0] as char {
                        'r' => {
                            if let Ok(mut p_term) = p_term_3.lock() {
                                if let Ok(pwd) = p_term.get_process_pwd() {
                                    p_term.respawn(&pwd);
                                }                            
                            }
                        }
                        'q' => {
                            if let Ok(mut p_term) = p_term_3.lock() {
                                p_term.join_handler = true;
                            }
                            break;
                        },
                        '\n' => {},
                        'i' => {
                            log_message("test");
                            escaped = false;
                        },
                        _ => {}
                    }        
                    let _ = stdin.read(&mut *key_buffer);
                } else {
                    while child_2.lock().is_ok_and(|mut c| c.try_wait().is_ok_and(|r| r.is_none())) {
                        if key_buffer[0] == 29 {
                            escaped = true;
                            break;
                        } else {
                            if let Ok(mut writer) = writer_2.lock() {
                                let _ = writer.write(&*key_buffer);
                                //write!(writer, "{}", buffer[0]).expect("oOF");
                            };
                        }
                        let _ = stdin.read(&mut *key_buffer);
                    }
                }
            }
        });


        Ok(p_term)
    }


    pub fn get_process_pwd(&self) -> io::Result<String> {
        if let Ok(child) = self.child.lock() {
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

        if let Ok(mut child) = self.child.lock() {
            child.kill()?;
            *child = match self.pty_pair.slave.spawn_command(cmd) {
                Ok(child) => {child},
                Err(_) => {
                    return Err(Error::new(io::ErrorKind::Other, "failed to spawn process"))}
            };
        }
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
                    esc.send(&mut self.writer.lock())?;
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
        self.join_handler = true;
        raw_mode(self.raw_mode);
        self.queue(Sequence::Escape(vec![Escape::ExitAltScreen]));
        self.flush();
    }

}
