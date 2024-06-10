extern crate fi;
use std::io::Read;

use fi::ascii::parser::*;
use fi::ascii::escapes::*;
use fi::pty::forker::*;
fn main() {
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

    old_main();
}
