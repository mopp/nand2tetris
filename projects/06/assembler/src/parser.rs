use std::io::prelude::*;
use std::io::BufRead;
use std::io::SeekFrom;

// trait Parser {
//     fn has_more_commands(&self) -> bool;
//     fn advance(&self);
//     fn command_type(&self) -> CommandType;
//     fn symbol(&self) -> String;
//     fn dest(&self) -> String;
//     fn comp(&self) -> String;
//     fn jump(&self) -> String;
// }

#[derive(Debug, PartialEq, Eq)]
enum CommandType {
    Address,
    Compute,
    Label,
}

#[derive(Debug)]
struct Parser<'a, T: BufRead + Seek> {
    contents: &'a mut T,
    position: usize,
    current_line: String,
    has_next: bool,
}

impl<'a, T: BufRead + Seek> Parser<'a, T> {
    fn new(contents: &'a mut T) -> Parser<'a, T> {
        let mut current_line = String::with_capacity(128);

        let num_bytes = contents
            .read_line(&mut current_line)
            .expect("reading won't fail");

        Parser {
            contents,
            position: current_line.len(),
            current_line,
            has_next: num_bytes != 0,
        }
    }

    fn has_more_commands(&self) -> bool {
        self.has_next
    }

    fn advance(&mut self) {
        debug_assert!(true, self.has_more_commands());

        self.current_line.clear();
        let num_bytes = self
            .contents
            .read_line(&mut self.current_line)
            .expect("reading won't fail");

        self.has_next = num_bytes != 0;
    }

    fn command_type(&self) -> CommandType {
        match self.current_line.trim().as_bytes()[0] {
            b'@' => CommandType::Address,
            b'(' => CommandType::Label,
            _ => CommandType::Compute,
        }
    }

    fn symbol(&self) -> String {
        unimplemented!();
    }

    fn dest(&self) -> String {
        unimplemented!();
    }

    fn comp(&self) -> String {
        unimplemented!();
    }

    fn jump(&self) -> String {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_has_more_commands() {
        let mut cursor = Cursor::new("D=A");
        let parser = Parser::new(&mut cursor);
        assert_eq!(true, parser.has_more_commands());

        // Empty input is given.
        let mut cursor = Cursor::new("");
        let parser = Parser::new(&mut cursor);
        assert_eq!(false, parser.has_more_commands());
    }

    #[test]
    fn test_command_type() {
        let mut cursor = Cursor::new(b"@999\n(LOOP)\nD=A");
        let mut parser = Parser::new(&mut cursor);
        assert_eq!(CommandType::Address, parser.command_type());

        parser.advance();
        assert_eq!(CommandType::Label, parser.command_type());

        parser.advance();
        assert_eq!(CommandType::Compute, parser.command_type());
    }
}
