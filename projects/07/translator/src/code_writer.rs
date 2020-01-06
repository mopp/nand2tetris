use super::parser::{Command, Segment};
use std::io::prelude::*;
use std::io::Result;

pub struct CodeWriter<'a, W: Write> {
    target: &'a mut W,
    filename: Option<&'a str>,
}

impl<'a, W: Write> CodeWriter<'a, W> {
    pub fn new(target: &'a mut W) -> Self {
        Self {
            target,
            filename: None,
        }
    }

    pub fn set_filename(&mut self, filename: &'a str) {
        self.filename = Some(filename);
    }

    pub fn put(&mut self, command: &Command) -> Result<()> {
        use Command::*;
        let instructions = match command {
            Add => "// = add =====================\n\
                 @SP\n\
                 M=M-1  // *SP -= 1\n\
                 A=M    // D = **SP\n\
                 D=M\n\
                 @SP\n\
                 A=M-1\n\
                 M=D+M  // *(*SP - 1) += D\n\
                 // ===========================\n"
                .to_string(),
            Push(Segment::Constant, index) => format!(
                "// = push constant {:5} =====\n\
                 @{}\n\
                 D=A    // D = {}\n\
                 @SP\n\
                 A=M\n\
                 M=D    // **SP = D\n\
                 @SP\n\
                 M=M+1  // *SP += 1\n",
                index, index, index
            ),
            _ => unimplemented!("TODO"),
        };

        self.target.write_all(instructions.as_bytes())
    }
}
