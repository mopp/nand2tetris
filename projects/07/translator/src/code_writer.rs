use super::parser::{Command, Segment};
use std::io::prelude::*;
use std::io::Result;

pub struct CodeWriter<'a, W: Write> {
    target: &'a mut W,
    filename: Option<&'a str>,
    label_counter: usize,
}

impl<'a, W: Write> CodeWriter<'a, W> {
    pub fn new(target: &'a mut W) -> Self {
        Self {
            target,
            filename: None,
            label_counter: 0,
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
                 A=M\n\
                 D=M    // D = **SP\n\
                 @SP\n\
                 A=M-1\n\
                 M=M+D  // *(*SP - 1) += D\n\
                 // ===========================\n"
                .to_string(),
            Sub => "// = sub =====================\n\
                 @SP\n\
                 M=M-1  // *SP -= 1\n\
                 A=M\n\
                 D=M    // D = **SP\n\
                 @SP\n\
                 A=M-1\n\
                 M=M-D  // *(*SP - 1) -= D\n\
                 // ===========================\n"
                .to_string(),
            Neg => "// = neg =====================\n\
                 @SP\n\
                 A=M - 1\n\
                 M=!M    // **SP = !**SP\n\
                 M=M+1   // **SP += 1\n\
                 // ===========================\n"
                .to_string(),
            And => "// = and =====================\n\
                 @SP\n\
                 M=M-1  // *SP -= 1\n\
                 A=M\n\
                 D=M    // D = **SP\n\
                 @SP\n\
                 A=M-1\n\
                 M=M&D  // *(*SP - 1) &= D\n\
                 // ===========================\n"
                .to_string(),
            Or => "// = or ======================\n\
                 @SP\n\
                 M=M-1  // *SP -= 1\n\
                 A=M\n\
                 D=M    // D = **SP\n\
                 @SP\n\
                 A=M-1\n\
                 M=M|D  // *(*SP - 1) |= D\n\
                 // ===========================\n"
                .to_string(),
            Not => "// = not =====================\n\
                 @SP\n\
                 A=M - 1\n\
                 M=!M    // **SP = !**SP\n\
                 // ===========================\n"
                .to_string(),
            Eq => {
                self.label_counter += 1;
                let c = self.label_counter;
                format!(
                    "// = eq ======================\n\
                     @SP\n\
                     M=M-1  // *SP -= 1\n\
                     A=M\n\
                     D=M    // D = **SP\n\
                     @SP\n\
                     A=M-1\n\
                     D=M-D  // D = *(*SP - 1) - D\n\
                     @_LABEL{}_TRUE\n\
                     D;JEQ\n\
                     D=0\n\
                     @_LABEL{}_END\n\
                     D;JMP\n\
                     (_LABEL{}_TRUE)\n\
                     D=-1\n\
                     (_LABEL{}_END)\n\
                     @SP\n\
                     A=M-1\n\
                     M=D\n  // *(*SP - 1) = D\
                     // ===========================\n",
                    c, c, c, c
                )
            }
            Lt => {
                self.label_counter += 1;
                let c = self.label_counter;
                format!(
                    "// = lq ======================\n\
                     @SP\n\
                     M=M-1  // *SP -= 1\n\
                     A=M\n\
                     D=M    // D = **SP\n\
                     @SP\n\
                     A=M-1\n\
                     D=M-D  // D = *(*SP - 1) - D\n\
                     @_LABEL{}_TRUE\n\
                     D;JLT\n\
                     D=0\n\
                     @_LABEL{}_END\n\
                     D;JMP\n\
                     (_LABEL{}_TRUE)\n\
                     D=-1\n\
                     (_LABEL{}_END)\n\
                     @SP\n\
                     A=M-1\n\
                     M=D\n  // *(*SP - 1) = D\
                     // ===========================\n",
                    c, c, c, c
                )
            }
            Gt => {
                self.label_counter += 1;
                let c = self.label_counter;
                format!(
                    "// = gt ======================\n\
                     @SP\n\
                     M=M-1  // *SP -= 1\n\
                     A=M\n\
                     D=M    // D = **SP\n\
                     @SP\n\
                     A=M-1\n\
                     D=M-D  // D = *(*SP - 1) - D\n\
                     @_LABEL{}_TRUE\n\
                     D;JGT\n\
                     D=0\n\
                     @_LABEL{}_END\n\
                     D;JMP\n\
                     (_LABEL{}_TRUE)\n\
                     D=-1\n\
                     (_LABEL{}_END)\n\
                     @SP\n\
                     A=M-1\n\
                     M=D\n  // *(*SP - 1) = D\
                     // ===========================\n",
                    c, c, c, c
                )
            }
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
