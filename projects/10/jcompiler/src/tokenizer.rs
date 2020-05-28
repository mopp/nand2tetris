#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Identifier(String),
    IntegerConstant(u16),
    StringConstant(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Symbol {
    ParenthesLeft,
    ParenthesRight,
    BracketLeft,
    BracketRight,
    BraceLeft,
    BraceRight,
    Dot,
    Comma,
    SemiColon,
    Plus,
    Minus,
    Star,
    Slash,
    And,
    Or,
    Lt,
    Gt,
    Equal,
    Not,
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    current: &'a str,
}

// impl<'a> Iterator for Tokenizer<'a> {
//     type Item = Token;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         None
//     }
// }

impl<'a> Tokenizer<'a> {
    pub fn new(current: &'a str) -> Self {
        Self { current }
    }

    pub fn next(&mut self) -> Option<Token> {
        self.skip();

        if self.current.is_empty() {
            return None;
        }

        // Tokenize single character symbol.
        let first_byte = self.current.as_bytes()[0];
        let r = match first_byte {
            b'(' => Some(Token::Symbol(ParenthesLeft)),
            b')' => Some(Token::Symbol(ParenthesRight)),
            b'[' => Some(Token::Symbol(BracketLeft)),
            b']' => Some(Token::Symbol(BracketRight)),
            b'{' => Some(Token::Symbol(BraceLeft)),
            b'}' => Some(Token::Symbol(BraceRight)),
            b'.' => Some(Token::Symbol(Dot)),
            b',' => Some(Token::Symbol(Comma)),
            b';' => Some(Token::Symbol(SemiColon)),
            b'+' => Some(Token::Symbol(Plus)),
            b'-' => Some(Token::Symbol(Minus)),
            b'*' => Some(Token::Symbol(Star)),
            b'/' => Some(Token::Symbol(Slash)),
            b'&' => Some(Token::Symbol(And)),
            b'|' => Some(Token::Symbol(Or)),
            b'<' => Some(Token::Symbol(Lt)),
            b'>' => Some(Token::Symbol(Gt)),
            b'=' => Some(Token::Symbol(Equal)),
            b'~' => Some(Token::Symbol(Not)),
            _ => None,
        };

        if r.is_some() {
            self.current = &self.current[1..];
            return r;
        }

        // Tokenize string constant.
        if first_byte == b'"' {
            if let Some(pos) = self.current[1..].find('"') {
                let r = Some(Token::StringConstant(self.current[1..=pos].to_string()));
                self.current = &self.current[pos + 2..];
                return r;
            } else {
                // TODO: Use Error.
                panic!("string is not closed.")
            }
        }

        // Tokenize word.
        let pos = self.find_token_tail()?;
        let word = &self.current[0..pos];
        self.current = &self.current[pos..];

        use Keyword::*;
        use Symbol::*;
        match word {
            "class" => Some(Token::Keyword(Class)),
            "constructor" => Some(Token::Keyword(Constructor)),
            "function" => Some(Token::Keyword(Function)),
            "method" => Some(Token::Keyword(Method)),
            "field" => Some(Token::Keyword(Field)),
            "static" => Some(Token::Keyword(Static)),
            "var" => Some(Token::Keyword(Var)),
            "int" => Some(Token::Keyword(Int)),
            "char" => Some(Token::Keyword(Char)),
            "boolean" => Some(Token::Keyword(Boolean)),
            "void" => Some(Token::Keyword(Void)),
            "true" => Some(Token::Keyword(True)),
            "false" => Some(Token::Keyword(False)),
            "null" => Some(Token::Keyword(Null)),
            "this" => Some(Token::Keyword(This)),
            "let" => Some(Token::Keyword(Let)),
            "do" => Some(Token::Keyword(Do)),
            "if" => Some(Token::Keyword(If)),
            "else" => Some(Token::Keyword(Else)),
            "while" => Some(Token::Keyword(While)),
            "return" => Some(Token::Keyword(Return)),
            word => {
                if let Ok(integer) = word.parse::<u16>() {
                    Some(Token::IntegerConstant(integer))
                } else {
                    Some(Token::Identifier(word.to_string()))
                }
            }
        }
    }

    fn skip(&mut self) {
        loop {
            match self.current.as_bytes() {
                [b' ', ..] | [b'\t', ..] | [b'\n', ..] | [b'\r', ..] => {
                    // Skip some special characters.
                    self.current = &self.current[1..]
                }
                [b'/', b'/', ..] => {
                    // Remove one line comment.
                    if let Some(pos) = self.current.find('\n') {
                        self.current = &self.current[pos + 1..];
                    } else {
                        // TODO: Use Error.
                        panic!("invalid comment!")
                    }
                }
                [b'/', b'*', b'*', ..] => {
                    // Remove multi line comment.
                    if let Some(pos) = self.current.find("*/") {
                        self.current = &self.current[pos + 2..];
                    } else {
                        // TODO: Use Error.
                        panic!("invalid comment!")
                    }
                }
                _ => break,
            }
        }
    }

    fn find_token_tail(&self) -> Option<usize> {
        self.current
            .find(|c: char| !(c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        let src = "// This file is part of www.nand2tetris.org\n \
                   // and the book \"The Elements of Computing Systems\"\n\
                   // by Nisan and Schocken, MIT Press.\n\
                   // File name: projects/10/ArrayTest/Main.jack\n\
                   \n\
                   // (identical to projects/09/Average/Main.jack)\n\
                   \n\
                   /** Computes the average of a sequence of integers. */\n\
                   class Main {\n\
                       function void main() {\n\
                        do Output.printString(\"THE AVERAGE IS: \");\n\
                        return;\n\
                       }\n\
                   }\n\
                   ";

        let mut tokenizer = Tokenizer::new(src);
        use Keyword::*;
        use Symbol::*;
        assert_eq!(Some(Token::Keyword(Class)), tokenizer.next());
        assert_eq!(
            Some(Token::Identifier("Main".to_string())),
            tokenizer.next()
        );
        assert_eq!(Some(Token::Symbol(BraceLeft)), tokenizer.next());
        assert_eq!(Some(Token::Keyword(Function)), tokenizer.next());
        assert_eq!(Some(Token::Keyword(Void)), tokenizer.next());
        assert_eq!(
            Some(Token::Identifier("main".to_string())),
            tokenizer.next()
        );
        assert_eq!(Some(Token::Symbol(ParenthesLeft)), tokenizer.next());
        assert_eq!(Some(Token::Symbol(ParenthesRight)), tokenizer.next());
        assert_eq!(Some(Token::Symbol(BraceLeft)), tokenizer.next());
        assert_eq!(Some(Token::Keyword(Do)), tokenizer.next());
        assert_eq!(
            Some(Token::Identifier("Output".to_string())),
            tokenizer.next()
        );
        assert_eq!(Some(Token::Symbol(Dot)), tokenizer.next());
        assert_eq!(
            Some(Token::Identifier("printString".to_string())),
            tokenizer.next()
        );
        assert_eq!(Some(Token::Symbol(ParenthesLeft)), tokenizer.next());
        assert_eq!(
            Some(Token::StringConstant("THE AVERAGE IS: ".to_string())),
            tokenizer.next()
        );
        assert_eq!(Some(Token::Symbol(ParenthesRight)), tokenizer.next());
        assert_eq!(Some(Token::Symbol(SemiColon)), tokenizer.next());
        assert_eq!(Some(Token::Keyword(Return)), tokenizer.next());
        assert_eq!(Some(Token::Symbol(SemiColon)), tokenizer.next());
        assert_eq!(Some(Token::Symbol(BraceRight)), tokenizer.next());
        assert_eq!(Some(Token::Symbol(BraceRight)), tokenizer.next());
        assert_eq!(None, tokenizer.next());
    }
}
