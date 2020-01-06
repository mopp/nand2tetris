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

    pub fn write_arithmetic(&mut self, command: &str) -> Result<()> {
        // NOTE: make the SP indicate the next available memory.
        let instructions = match command {
            "add" => {
                "// = add =====================\n\
                 @SP\n\
                 M=M-1  // *SP -= 1\n\
                 A=M    // D = **SP\n\
                 D=M\n\
                 @SP\n\
                 A=M-1\n\
                 M=D+M  // *(*SP - 1) += D\n\
                 // ===========================\n"
            }
            _ => "",
        };

        self.target.write_all(instructions.as_bytes())
    }

    pub fn write_push_pop(&mut self, command: &str, segment: &str, index: u16) -> Result<()> {
        let instructions = match (command, segment) {
            ("push", "constant") => format!(
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

    // pub fn close() {}
}
