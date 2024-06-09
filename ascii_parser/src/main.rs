use std::io::{self, Read};

#[derive(Debug)]
enum Sequence {
    Text(String),
    Escape(String),
}

fn main() {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer).expect("Failed to read input");

    let input = String::from_utf8_lossy(&buffer);
    let sequences = parse_sequences(&input);

    for sequence in sequences {
        match sequence {
            Sequence::Text(text) => println!("Text: {:?}", text),
            Sequence::Escape(esc) => println!("Escape: {:?}", esc),
        }
    }
}

fn parse_sequences(input: &str) -> Vec<Sequence> {
    let mut sequences = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '\x1b' {
            // Start of an escape sequence
            let mut escape_seq = String::new();
            escape_seq.push(chars.next().unwrap()); // consume '\x1b'

            if let Some(&'[') = chars.peek() {
                escape_seq.push(chars.next().unwrap()); // consume '['

                // Continue until a letter is found (end of the escape sequence)
                while let Some(&c) = chars.peek() {
                    escape_seq.push(chars.next().unwrap());
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            sequences.push(Sequence::Escape(escape_seq));
        } else {
            // Normal text
            let mut text = String::new();
            while let Some(&c) = chars.peek() {
                if c == '\x1b' {
                    break;
                } else {
                    text.push(chars.next().unwrap());
                }
            }
            sequences.push(Sequence::Text(text));
        }
    }

    sequences
}
