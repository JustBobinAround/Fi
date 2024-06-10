use std::borrow::Cow;

use super::escapes::*;


pub fn parse_sequences(input: Cow<str>) -> Vec<Sequence> {
    let mut chars = input.chars().peekable();

    let sequences = Sequence::parse_sequence(&mut chars);

    sequences
}


