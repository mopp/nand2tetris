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
                 M=M+1  // *SP += 1\n\
                 // ===========================\n",
                index, index, index
            ),
            Push(Segment::Temp, index) => format!(
                "// = push temp {:2} =====\n\
                 @5\n\
                 D=A\n\
                 @{}\n\
                 A=A+D\n\
                 D=M    // D = temp[{}]\n\
                 @SP\n\
                 M=M+1  // *SP += 1\n\
                 A=M-1\n\
                 M=D    // *(*SP - 1) = D\n\
                 // ===========================\n",
                index, index, index
            ),
            Push(Segment::Static, _) => unimplemented!("TODO"),
            Push(Segment::Pointer, _) => unimplemented!("TODO"),
            Push(segment, index) => {
                let register_name = self.segment_to_register(segment);
                format!(
                    "// = push {:<8} {:5} =========\n\
                     @{}\n\
                     D=M\n\
                     @{}\n\
                     A=D+A\n\
                     D=M    // D = &{}[{}]\n\
                     @SP\n\
                     M=M+1  // *SP += 1\n\
                     A=M-1\n\
                     M=D    // *(*SP - 1) = D\n\
                     // ===========================\n",
                    segment, index, register_name, index, segment, index
                )
            }
            Pop(Segment::Temp, index) => format!(
                "// = pop temp {:2} ===========\n\
                 @5\n\
                 D=A\n\
                 @{}\n\
                 D=D+A\n\
                 @R13\n\
                 M=D    // R13 = &temp[{}]\n\
                 @SP\n\
                 M=M-1  // *SP -= 1\n\
                 A=M\n\
                 D=M    // D = **SP\n\
                 @R13\n\
                 A=M\n\
                 M=D    // *R13 = D\n\
                 // ===========================\n",
                index, index, index
            ),
            Pop(segment, index) => {
                let register_name = self.segment_to_register(segment);
                format!(
                    "// = pop {:<8} {:5} ======\n\
                     @{}\n\
                     D=M\n\
                     @{}\n\
                     D=D+A\n\
                     @R13\n\
                     M=D    // R13 = &{}[{}]\n\
                     @SP\n\
                     M=M-1  // *SP -= 1\n\
                     A=M\n\
                     D=M    // D = **SP\n\
                     @R13\n\
                     A=M\n\
                     M=D    // *R13 = D\n\
                     // ===========================\n",
                    segment, index, register_name, index, segment, index
                )
            }
            cmd => unimplemented!("TODO: {:?}", cmd),
        };

        self.target.write_all(instructions.as_bytes())
    }

    fn segment_to_register(&self, segment: &Segment) -> &'static str {
        match segment {
            Segment::Local => "LCL",
            Segment::Argument => "ARG",
            Segment::This => "THIS",
            Segment::That => "THAT",
            Segment::Temp => panic!("internal error"),
            _ => unimplemented!("TODO"),
        }
    }
}
