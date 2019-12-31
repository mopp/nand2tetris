use std::io::BufRead;

#[derive(Debug, PartialEq, Eq)]
pub enum CommandType {
    Address,
    Compute,
    Label,
}

#[derive(Debug)]
pub struct Parser<'a, T: BufRead> {
    contents: &'a mut T,
    current_line: String,
    has_next: bool,
}

impl<'a, T: BufRead> Parser<'a, T> {
    pub fn new(contents: &'a mut T) -> Parser<'a, T> {
        let current_line = String::with_capacity(128);

        let mut p = Parser {
            contents,
            current_line,
            has_next: true,
        };

        p.advance();

        p
    }

    pub fn has_more_commands(&self) -> bool {
        self.has_next
    }

    pub fn advance(&mut self) {
        if !self.has_more_commands() {
            panic!("You cannot call advance if has_more_commands returns false");
        }

        loop {
            self.current_line.clear();
            let num_bytes = self
                .contents
                .read_line(&mut self.current_line)
                .expect("reading won't fail");

            if num_bytes == 0 {
                // EOF.
                self.has_next = false;
                break;
            }

            // Skip empty lines.
            self.current_line.retain(|c| !c.is_whitespace());
            if !self.current_line.is_empty() && self.current_line.as_bytes()[0] != b'/' {
                self.has_next = true;
                break;
            }
        }
    }

    pub fn command_type(&self) -> CommandType {
        match self.current_line.as_bytes()[0] {
            b'@' => CommandType::Address,
            b'(' => CommandType::Label,
            _ => CommandType::Compute,
        }
    }

    pub fn symbol(&self) -> String {
        match self.command_type() {
            CommandType::Address => {
                let s = self.current_line.as_str();
                let end = s.len();
                s[1..end].to_string()
            }
            CommandType::Label => {
                let s = self.current_line.as_str();
                let end = s.len();
                s[1..end - 1].to_string()
            }
            CommandType::Compute => {
                panic!("You can call symbol only if the command type is address or label")
            }
        }
    }

    pub fn dest(&self) -> Option<String> {
        if self.command_type() != CommandType::Compute {
            panic!("You can call dest only if the command type is compute");
        }

        if let Some(i) = self.current_line.find('=') {
            Some(self.current_line[0..i].to_string())
        } else {
            None
        }
    }

    pub fn comp(&self) -> String {
        if self.command_type() != CommandType::Compute {
            panic!("You can call dest only if the command type is compute");
        }

        let i_head = if let Some(i) = self.current_line.find('=') {
            i + 1
        } else {
            0
        };

        let i_tail = if let Some(i) = self.current_line.find(';') {
            i
        } else {
            self.current_line.len()
        };

        self.current_line[i_head..i_tail].to_string()
    }

    pub fn jump(&self) -> Option<String> {
        if self.command_type() != CommandType::Compute {
            panic!("You can call dest only if the command type is compute");
        }

        if let Some(i) = self.current_line.find(';') {
            Some(self.current_line[i + 1..self.current_line.len()].to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn has_more_commands_test() {
        let mut cursor = Cursor::new("D=A");
        let parser = Parser::new(&mut cursor);
        assert_eq!(true, parser.has_more_commands());

        // Empty input is given.
        let mut cursor = Cursor::new("");
        let parser = Parser::new(&mut cursor);
        assert_eq!(false, parser.has_more_commands());
    }

    #[test]
    fn command_type_test() {
        let mut cursor = Cursor::new(b"@999\n(LOOP)\nD=A");
        let mut parser = Parser::new(&mut cursor);
        assert_eq!(CommandType::Address, parser.command_type());

        parser.advance();
        assert_eq!(CommandType::Label, parser.command_type());

        parser.advance();
        assert_eq!(CommandType::Compute, parser.command_type());
    }

    #[test]
    fn symbol_test() {
        let mut cursor = Cursor::new(b"@999\n(LOOP)\nD=A");
        let mut parser = Parser::new(&mut cursor);
        assert_eq!("999", parser.symbol());

        parser.advance();
        assert_eq!("LOOP", parser.symbol());
    }

    #[test]
    fn dest_test() {
        let mut cursor = Cursor::new(b"D=A");
        let parser = Parser::new(&mut cursor);
        assert_eq!(Some("D".to_string()), parser.dest());

        let mut cursor = Cursor::new(b" AMD =A");
        let parser = Parser::new(&mut cursor);
        assert_eq!(Some("AMD".to_string()), parser.dest());

        let mut cursor = Cursor::new(b"0;JMP");
        let parser = Parser::new(&mut cursor);
        assert_eq!(None, parser.dest());
    }

    #[test]
    fn comp_test() {
        let mut cursor = Cursor::new(b"0;JMP");
        let parser = Parser::new(&mut cursor);
        assert_eq!("0", parser.comp());

        let mut cursor = Cursor::new(b"A = M - 1");
        let parser = Parser::new(&mut cursor);
        assert_eq!("M-1", parser.comp());
    }

    #[test]
    fn jump_test() {
        let mut cursor = Cursor::new(b"0;JMP");
        let parser = Parser::new(&mut cursor);
        assert_eq!(Some("JMP".to_string()), parser.jump());

        let mut cursor = Cursor::new(b"D ; JEQ ");
        let parser = Parser::new(&mut cursor);
        assert_eq!(Some("JEQ".to_string()), parser.jump());
    }
}
