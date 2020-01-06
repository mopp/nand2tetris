use std::io::BufRead;

#[derive(Debug, PartialEq, Eq)]
pub enum Segment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

pub type Index = u16;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Push(Segment, Index),
    Pop(Segment, Index),
    Label,
    Goto,
    If,
    Function,
    Return,
    Call,
}

#[derive(Debug)]
pub struct Parser<'a, T: BufRead> {
    contents: &'a mut T,
    buf: String,
}

impl<'a, T: BufRead> Parser<'a, T> {
    pub fn new(contents: &'a mut T) -> Self {
        Self {
            contents,
            buf: String::with_capacity(512),
        }
    }

    fn parse_segment<'b>(segment: &'b str) -> Segment {
        match segment {
            "argument" => Segment::Argument,
            "local" => Segment::Local,
            "static" => Segment::Static,
            "constant" => Segment::Constant,
            "this" => Segment::This,
            "that" => Segment::That,
            "pointer" => Segment::Pointer,
            "temp" => Segment::Temp,
            _ => panic!("Invalid segment: [{}]", segment),
        }
    }
}

impl<'a, T: BufRead> Iterator for Parser<'a, T> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();

            if self
                .contents
                .read_line(&mut self.buf)
                .expect("reading won't fail")
                == 0
            {
                // EOF.
                return None;
            }

            // Remove newline.
            self.buf.retain(|c| c != '\n' && c != '\r');

            // Remove comment.
            match self.buf.find("//") {
                Some(0) => continue,
                Some(i) => self.buf.truncate(i),
                None => {}
            }

            if !self.buf.is_empty() {
                break;
            }
        }

        let command = self.buf.split_whitespace().collect::<Vec<&str>>();

        Some(match command[0] {
            "add" => Command::Add,
            "sub" => Command::Sub,
            "neg" => Command::Neg,
            "eq" => Command::Eq,
            "gt" => Command::Gt,
            "lt" => Command::Lt,
            "and" => Command::And,
            "or" => Command::Or,
            "not" => Command::Not,
            "push" => Command::Push(
                Parser::<T>::parse_segment(command[1]),
                command[2]
                    .parse::<u16>()
                    .expect("The 2nd argument cannot be parse"),
            ),
            "pop" => Command::Pop(
                Parser::<T>::parse_segment(command[1]),
                command[2]
                    .parse::<u16>()
                    .expect("The 2nd argument cannot be parse"),
            ),
            "label" => Command::Label,
            "goto" => Command::Goto,
            "if-goto" => Command::If,
            "function" => Command::Function,
            "return" => Command::Return,
            "call" => Command::Call,
            command => panic!("Invalid command: [{}]", command),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const SAMPLE_VM_CODE1: &str =
        "// File name: projects/07/MemoryAccess/PointerTest/PointerTest.vm\n\
         \n\
         // Executes pop and push commands using the \n\
         // pointer, this, and that segments.\n\
         push constant 3030\n\
         pop pointer 0\n\
         push constant 3040";
    const SAMPLE_VM_CODE2: &str = "add\n\
                                   sub\n\
                                   push constant 3040";

    #[test]
    fn has_more_commands_test() {
        let mut cursor = Cursor::new(SAMPLE_VM_CODE1);
        let parser = Parser::new(&mut cursor);

        assert_eq!(
            vec![
                Command::Push(Segment::Constant, 3030),
                Command::Pop(Segment::Pointer, 0),
                Command::Push(Segment::Constant, 3040)
            ],
            parser.collect::<Vec<_>>()
        );

        let mut cursor = Cursor::new(SAMPLE_VM_CODE2);
        let parser = Parser::new(&mut cursor);

        assert_eq!(
            vec![
                Command::Add,
                Command::Sub,
                Command::Push(Segment::Constant, 3040),
            ],
            parser.collect::<Vec<_>>()
        );
    }
}
