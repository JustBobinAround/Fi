use libc::termios as Termios;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtyPair, PtySize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{self, Error, Read, Stdin, Stdout, Write};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use super::raw_mode::raw_mode;
use crate::ascii::escapes::{self, Escape, EscapeWriter, Sequence};

pub struct PTerminal{
    writer: Stdout,
    to_write: Vec<u8>,
    raw_mode: Option<Termios>,
    pub join_handler: bool,
    key_buffer: Arc<Mutex<[u8; 1]>>,
    size_x: u16,
    size_y: u16,
    offset_x: u32,
    offset_y: u32,
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


        let mut child = match pair.slave.spawn_command(cmd) {
            Ok(child) => {child},
            Err(_) => {return Err(Error::new(io::ErrorKind::Other, "failed to spawn process"))}
        };

        let reader = Arc::new(Mutex::new(pair.master.try_clone_reader().expect("OOF")));
        let reader_2 = reader.clone();
        let writer = Arc::new(Mutex::new(pair.master.take_writer().expect("OOF")));
        let writer_2 = writer.clone();
        let to_write = Vec::new();
        let key_buffer = Arc::new(Mutex::new([0;1]));
        let key_buffer_2 = key_buffer.clone();

        let p_term = Arc::new(Mutex::new(PTerminal { 
            writer: io::stdout(),
            to_write,
            raw_mode,
            key_buffer,
            join_handler: false,
            size_x,
            size_y,
            offset_x,
            offset_y,
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
            let mut s: [u8; 1] = [0;1];

            while let Ok(mut reader) = reader_2.lock() {
                if p_term_2.lock().is_ok_and(|j|j.join_handler) {
                    break;
                }
                match reader.read_exact(&mut s) {
                    Ok(n) => {
                        if let Ok(mut p_term) = p_term_2.lock() {
                            p_term.to_write.push(s[0]);
                            p_term.flush();
                        }
                    },
                    Err(_) => {}
                }
            };
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
                        'q' => {
                            if let Ok(mut p_term) = p_term_3.lock() {
                                p_term.join_handler = true;
                            }
                            break;
                        },
                        '\n' => {},
                        'i' => {
                            escaped = false;
                        },
                        _ => {}
                    }        
                    let _ = stdin.read(&mut *key_buffer);
                } else {
                    while child.try_wait().is_ok_and(|r| r.is_none())  {
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
