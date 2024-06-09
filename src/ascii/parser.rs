use std::{io::{self, Read}, borrow::Cow};
use super::escapes::*;


pub fn parse_sequences<'a>(input: &'a Cow<'a, str>) -> Vec<Sequence<'a>> {
    let mut chars = input.chars().peekable();

    let sequences = Sequence::parse_sequence(input, &mut chars);

    sequences
}

