extern crate fi;
use std::io;
use std::io::Read;
use std::thread;
use std::time::Duration;

use fi::ascii::parser::*;
use fi::ascii::escapes::*;
use fi::pty::forker::*;
use portable_pty::CommandBuilder;
fn main() -> io::Result<()>{
    let mut buffer = Vec::new();
    //std::io::stdin().read_to_end(&mut buffer).expect("Failed to read input");

    let input = String::from_utf8_lossy(&buffer);
    let sequences = parse_sequences(input);

    for sequence in sequences {
        match sequence {
            Sequence::Text(text) => println!("Text: {:?}", text),
            Sequence::Escape(esc) => println!("Escape: {:?}", esc),
            _ => {}
        }
    }

    let p_term = PTerminal::new(80, 40, 0, 0)?;

    while p_term.lock().is_ok_and(|j|!j.join_handler) {
        thread::sleep(Duration::from_millis(10));
    }

    if let Ok(mut p_term) = p_term.lock() {
        p_term.close();
    }

    Ok(())
}
