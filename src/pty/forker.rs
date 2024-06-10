use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::sync::{Arc, Mutex};
use std::io::{self, Write, Read};
use super::raw_mode::raw_mode;
use crate::ascii::escapes::{Escape, EscapeWriter};

pub fn old_main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();
    let mut escapesWr = EscapeWriter::new(&mut stdout);

    raw_mode(true)?;
    escapesWr.queue(Escape::EnterAltScreen);
    escapesWr.queue(Escape::ClearAll);

    escapesWr.send_all_and_flush()?;

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
        let mut s: [u8; 1] = [0;1];
        let mut stdout = io::stdout();
        let mut last_x = 0;
        let mut last_y = 0;

        let mut was_escaped = false;
        while let Ok(mut reader) = reader.lock() {
            let n = reader.read_exact(&mut s).unwrap();

            //stdout.queue(cursor::MoveTo(last_x,last_y));
            let _ = stdout.write(&s);
            stdout.flush();
        };
        return;
    });

    escapesWr.flush()?;

    loop {
        if escaped.lock().is_ok_and(|b| *b==true) {
            if buffer[0] == 29 {
                if let Ok(mut escaped) = escaped.lock() {
                    *escaped = true;
                };
            }
            escapesWr.flush()?;
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
            escapesWr.flush();
            stdin.read(&mut buffer)?;
        } else {
//            stdout
//                .queue(cursor::MoveTo(0,23))?
//                .queue(style::Print("-- INSERT --"))?
//                .queue(cursor::MoveTo(0,0))?;
            while child.try_wait().is_ok_and(|r| r.is_none())  {
                escapesWr.flush();
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
                escapesWr.flush();
                stdin.read(&mut buffer)?;
            }
        }
    }

    raw_mode(false)?;

    escapesWr.queue(Escape::ExitAltScreen);
    escapesWr.send_all_and_flush()?;
    return Ok(());
}

