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

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    StringNotClosed,
    MultiLineCommentNotClosed,
    OneLineCommentNotClosed,
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    current: &'a str,
    current_token: Option<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(current: &'a str) -> Self {
        Self {
            current,
            current_token: None,
        }
    }

    pub fn get_current_token(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    pub fn advance(&mut self) -> Result<Option<&Token>, Error> {
        self.skip()?;

        if self.current.is_empty() {
            self.current_token = None;
            return Ok(None);
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
            self.current_token = r;
            return Ok(self.current_token.as_ref());
        }

        // Tokenize string constant.
        if first_byte == b'"' {
            if let Some(pos) = self.current[1..].find('"') {
                let r = Some(Token::StringConstant(self.current[1..=pos].to_string()));
                self.current = &self.current[pos + 2..];
                self.current_token = r;
                return Ok(self.current_token.as_ref());
            } else {
                // TODO: Use Error.
                return Err(Error::StringNotClosed);
            }
        }

        // Tokenize word.
        let pos = if let Some(pos) = self.find_token_tail() {
            pos
        } else {
            panic!("unexpected error: cannot find token");
        };

        let word = &self.current[0..pos];
        self.current = &self.current[pos..];

        use Keyword::*;
        use Symbol::*;
        let token = match word {
            "class" => Token::Keyword(Class),
            "constructor" => Token::Keyword(Constructor),
            "function" => Token::Keyword(Function),
            "method" => Token::Keyword(Method),
            "field" => Token::Keyword(Field),
            "static" => Token::Keyword(Static),
            "var" => Token::Keyword(Var),
            "int" => Token::Keyword(Int),
            "char" => Token::Keyword(Char),
            "boolean" => Token::Keyword(Boolean),
            "void" => Token::Keyword(Void),
            "true" => Token::Keyword(True),
            "false" => Token::Keyword(False),
            "null" => Token::Keyword(Null),
            "this" => Token::Keyword(This),
            "let" => Token::Keyword(Let),
            "do" => Token::Keyword(Do),
            "if" => Token::Keyword(If),
            "else" => Token::Keyword(Else),
            "while" => Token::Keyword(While),
            "return" => Token::Keyword(Return),
            word => {
                if let Ok(integer) = word.parse::<u16>() {
                    Token::IntegerConstant(integer)
                } else {
                    Token::Identifier(word.to_string())
                }
            }
        };

        self.current_token = Some(token);
        Ok(self.current_token.as_ref())
    }

    fn skip(&mut self) -> Result<(), Error> {
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
                        return Err(Error::OneLineCommentNotClosed);
                    }
                }
                [b'/', b'*', b'*', ..] => {
                    // Remove multi line comment.
                    if let Some(pos) = self.current.find("*/") {
                        self.current = &self.current[pos + 2..];
                    } else {
                        return Err(Error::MultiLineCommentNotClosed);
                    }
                }
                _ => break,
            }
        }

        Ok(())
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

        assert_eq!(None, tokenizer.get_current_token());

        assert_eq!(Ok(Some(&Token::Keyword(Class))), tokenizer.advance());
        assert_eq!(Some(&Token::Keyword(Class)), tokenizer.get_current_token());

        assert_eq!(
            Ok(Some(&Token::Identifier("Main".to_string()))),
            tokenizer.advance()
        );
        assert_eq!(
            Some(&Token::Identifier("Main".to_string())),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Symbol(BraceLeft))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Symbol(BraceLeft)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Keyword(Function))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Keyword(Function)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Keyword(Void))), tokenizer.advance());
        assert_eq!(Some(&Token::Keyword(Void)), tokenizer.get_current_token());

        assert_eq!(
            Ok(Some(&Token::Identifier("main".to_string()))),
            tokenizer.advance()
        );
        assert_eq!(
            Some(&Token::Identifier("main".to_string())),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Symbol(ParenthesLeft))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Symbol(ParenthesLeft)),
            tokenizer.get_current_token()
        );

        assert_eq!(
            Ok(Some(&Token::Symbol(ParenthesRight))),
            tokenizer.advance()
        );
        assert_eq!(
            Some(&Token::Symbol(ParenthesRight)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Symbol(BraceLeft))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Symbol(BraceLeft)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Keyword(Do))), tokenizer.advance());
        assert_eq!(Some(&Token::Keyword(Do)), tokenizer.get_current_token());

        assert_eq!(
            Ok(Some(&Token::Identifier("Output".to_string()))),
            tokenizer.advance()
        );
        assert_eq!(
            Some(&Token::Identifier("Output".to_string())),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Symbol(Dot))), tokenizer.advance());
        assert_eq!(Some(&Token::Symbol(Dot)), tokenizer.get_current_token());

        assert_eq!(
            Ok(Some(&Token::Identifier("printString".to_string()))),
            tokenizer.advance()
        );
        assert_eq!(
            Some(&Token::Identifier("printString".to_string())),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Symbol(ParenthesLeft))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Symbol(ParenthesLeft)),
            tokenizer.get_current_token()
        );

        assert_eq!(
            Ok(Some(&Token::StringConstant("THE AVERAGE IS: ".to_string()))),
            tokenizer.advance()
        );
        assert_eq!(
            Some(&Token::StringConstant("THE AVERAGE IS: ".to_string())),
            tokenizer.get_current_token()
        );

        assert_eq!(
            Ok(Some(&Token::Symbol(ParenthesRight))),
            tokenizer.advance()
        );
        assert_eq!(
            Some(&Token::Symbol(ParenthesRight)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Symbol(SemiColon))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Symbol(SemiColon)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Keyword(Return))), tokenizer.advance());
        assert_eq!(Some(&Token::Keyword(Return)), tokenizer.get_current_token());

        assert_eq!(Ok(Some(&Token::Symbol(SemiColon))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Symbol(SemiColon)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Symbol(BraceRight))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Symbol(BraceRight)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(Some(&Token::Symbol(BraceRight))), tokenizer.advance());
        assert_eq!(
            Some(&Token::Symbol(BraceRight)),
            tokenizer.get_current_token()
        );

        assert_eq!(Ok(None), tokenizer.advance());
        assert_eq!(None, tokenizer.get_current_token());
    }
}
