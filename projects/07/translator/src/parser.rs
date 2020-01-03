use std::io::BufRead;
use std::io::{Error, ErrorKind};

#[derive(Debug, PartialEq, Eq)]
pub enum CommandType {
    Arithmetic,
    Push,
    Pop,
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
    current_line: String,
    next_line: Option<String>,
}

impl<'a, T: BufRead> Parser<'a, T> {
    pub fn new(contents: &'a mut T) -> Result<Self, Error> {
        let current_line =
            Self::read_line(contents).ok_or_else(|| Error::new(ErrorKind::NotFound, "No input"))?;
        let next_line = Self::read_line(contents);

        Ok(Self {
            contents,
            current_line,
            next_line,
        })
    }

    pub fn has_more_commands(&self) -> bool {
        self.next_line.is_some()
    }

    pub fn advance(&mut self) {
        if !self.has_more_commands() {
            panic!("You cannot call advance if has_more_commands returns false");
        }

        self.current_line = self.next_line.take().unwrap();
        self.next_line = Parser::read_line(self.contents);
    }

    fn read_line<'b>(contents: &'b mut T) -> Option<String> {
        let mut buf = String::new();

        loop {
            buf.clear();

            if contents.read_line(&mut buf).expect("reading won't fail") == 0 {
                // EOF.
                return None;
            }

            // Remove newline.
            buf.retain(|c| c != '\n');

            // Remove comment.
            match buf.find("//") {
                Some(0) => continue,
                Some(i) => buf.truncate(i),
                None => {}
            }

            if !buf.is_empty() {
                break;
            }
        }

        Some(buf)
    }

    pub fn command_type(&self) -> CommandType {
        let command = if let Some(i) = self.current_line.find(' ') {
            &self.current_line[0..i]
        } else {
            // Arithmetic command has no arguments.
            self.current_line.as_str()
        };

        match command {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => {
                CommandType::Arithmetic
            }
            "push" => CommandType::Push,
            "pop" => CommandType::Pop,
            "label" => CommandType::Label,
            "goto" => CommandType::Goto,
            "if-goto" => CommandType::If,
            "function" => CommandType::Function,
            "return" => CommandType::Return,
            "call" => CommandType::Call,
            command => panic!("Invalid command: {}", command),
        }
    }

    pub fn arg1(&self) -> String {
        let command = self.current_line.split_whitespace().collect::<Vec<_>>();
        match self.command_type() {
            CommandType::Arithmetic => command[0].to_string(),
            CommandType::Return => panic!("You cannot call arg1 when the command type is return"),
            _ => command[1].to_string(),
        }
    }

    pub fn arg2(&self) -> u16 {
        let command = self.current_line.split_whitespace().collect::<Vec<_>>();
        match self.command_type() {
            CommandType::Push | CommandType::Pop | CommandType::Function | CommandType::Call => {
                command[2].parse::<u16>().expect("The 2nd argument cannot be parse")
            },
            _ => panic!(
                "You can call arg2 only when the command type is push, pop, function, or call"
            ),
        }
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
        let mut parser = Parser::new(&mut cursor).unwrap();

        assert_eq!(true, parser.has_more_commands());
        assert_eq!("push constant 3030", parser.current_line);
        parser.advance();
        assert_eq!(true, parser.has_more_commands());
        assert_eq!("pop pointer 0", parser.current_line);
        parser.advance();
        assert_eq!(false, parser.has_more_commands());
        assert_eq!("push constant 3040", parser.current_line);
    }

    #[test]
    fn command_type_test() {
        let mut cursor = Cursor::new(SAMPLE_VM_CODE1);
        let mut parser = Parser::new(&mut cursor).unwrap();

        assert_eq!(CommandType::Push, parser.command_type());
        parser.advance();
        assert_eq!(CommandType::Pop, parser.command_type());
        parser.advance();
        assert_eq!(CommandType::Push, parser.command_type());

        let mut cursor = Cursor::new(SAMPLE_VM_CODE2);
        let mut parser = Parser::new(&mut cursor).unwrap();

        assert_eq!(CommandType::Arithmetic, parser.command_type());
        parser.advance();
        assert_eq!(CommandType::Arithmetic, parser.command_type());
        parser.advance();
        assert_eq!(CommandType::Push, parser.command_type());
    }

    #[test]
    fn arg1_test() {
        let mut cursor = Cursor::new(SAMPLE_VM_CODE1);
        let mut parser = Parser::new(&mut cursor).unwrap();

        assert_eq!("constant", parser.arg1().as_str());
        parser.advance();
        assert_eq!("pointer", parser.arg1().as_str());
        parser.advance();
        assert_eq!("constant", parser.arg1().as_str());

        let mut cursor = Cursor::new(SAMPLE_VM_CODE2);
        let mut parser = Parser::new(&mut cursor).unwrap();

        assert_eq!("add", parser.arg1().as_str());
        parser.advance();
        assert_eq!("sub", parser.arg1().as_str());
        parser.advance();
        assert_eq!("constant", parser.arg1().as_str());
    }

    #[test]
    fn arg2_test() {
        let mut cursor = Cursor::new(SAMPLE_VM_CODE1);
        let mut parser = Parser::new(&mut cursor).unwrap();

        assert_eq!(3030, parser.arg2());
        parser.advance();
        assert_eq!(0, parser.arg2());
        parser.advance();
        assert_eq!(3040, parser.arg2());
    }
}
