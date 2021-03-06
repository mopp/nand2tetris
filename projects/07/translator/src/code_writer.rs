use super::parser::{Command, Index, Indirect, MappedMemory, Segment};
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
        use MappedMemory::*;
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
            Push(Segment::Indirect(segment), index) => {
                let register_name = self.get_indirect_register_name(segment);
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
            Push(Segment::MappedMemory(Temp), index) => format!(
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
            Push(Segment::MappedMemory(Pointer), index) => format!(
                "// = push pointer {:2}  =========\n\
                 @{}
                 D=M    // D = pointer[{}]\n\
                 @SP
                 M=M+1  // *SP += 1\n\
                 A=M-1\n\
                 M=D    // **SP = D\n\
                 // ===========================\n",
                index,
                self.get_pointer_resigter_name(*index),
                index
            ),
            Push(Segment::Static, index) => format!(
                "// = push static {}  =========\n\
                     @{}.{}
                     D=M
                     @SP
                     M=M+1  // *SP += 1\n\
                     A=M-1\n\
                     M=D    // **SP = D\n\
                     // ===========================\n",
                index,
                self.filename.expect("no filename"),
                index
            ),
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
            Pop(Segment::Indirect(segment), index) => {
                let register_name = self.get_indirect_register_name(segment);
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
            Pop(Segment::MappedMemory(Temp), index) => format!(
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
            Pop(Segment::MappedMemory(Pointer), index) => format!(
                "// = pop pointer {:2}  =========\n\
                     @SP\n\
                     M=M-1  // *SP -= 1\n\
                     A=M\n\
                     D=M    // D = **SP\n\
                     @{}
                     M=D    // pointer[{}] = D\n\
                     // ===========================\n",
                index,
                self.get_pointer_resigter_name(*index),
                index
            ),
            Pop(Segment::Static, index) => format!(
                "// = pop static {}  =========\n\
                     @SP\n\
                     M=M-1  // *SP -= 1\n\
                     A=M\n\
                     D=M    // D = **SP\n\
                     @{}.{}
                     M=D    // pointer[{}] = D\n\
                     // ===========================\n",
                index,
                self.filename.expect("no filename"),
                index,
                index
            ),
            Pop(Segment::Constant, _) => panic!("pop constant N is invalid."),
            _ => unimplemented!("TODO: the other instructions"),
        };

        self.target.write_all(instructions.as_bytes())
    }

    fn get_indirect_register_name(&self, segment: &Indirect) -> &'static str {
        match segment {
            Indirect::Local => "LCL",
            Indirect::Argument => "ARG",
            Indirect::This => "THIS",
            Indirect::That => "THAT",
        }
    }

    fn get_pointer_resigter_name(&self, index: Index) -> &'static str {
        match index {
            0 => "THIS",
            1 => "THAT",
            _ => panic!("unexpected pointer segment: {}", index),
        }
    }
}
