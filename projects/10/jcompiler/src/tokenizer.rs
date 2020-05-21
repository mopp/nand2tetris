use std::io::prelude::*;
use std::io::Result;

pub struct Tokenizer<'a, R: Read> {
    src: &'a mut R,
    lines: Vec<String>,
    pos: usize,
}

pub enum Token {
    Keyword,
    Symbol,
    Identifier,
    IntConst,
    StringConst,
}

impl<'a, R: Read> Iterator for Tokenizer<'a, R> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a, R: Read> Tokenizer<'a, R> {
    pub fn new(src: &'a mut R) -> Result<Self> {
        let mut buf = String::new();
        src.read_to_string(&mut buf)?;

        Ok(Self {
            src,
            lines: buf.lines().map(|s| s.to_string()).collect::<Vec<String>>(),
            pos: 0
        })
    }
}
