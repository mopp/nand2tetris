use crate::tokenizer::{self, Keyword::*, Symbol, Token, Token::*};
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;

const INDENT_SIZE: usize = 2;

#[derive(Debug)]
pub struct Parser<W: Write> {
    tokens: Vec<Token>,
    current_index: usize,
    spaces: String,
    writer: BufWriter<W>,
}

impl<W: Write> Parser<W> {
    pub fn new(tokens: Vec<Token>, writer: W) -> Self {
        Self {
            tokens,
            current_index: 0,
            spaces: String::new(),
            writer: BufWriter::new(writer),
        }
    }

    pub fn compile(&mut self) -> Result<(), Error> {
        if &Keyword(Class) == self.advance()? {
            self.compile_class()
        } else {
            Err(Error::UnexpectedInput(
                "top level component have to be class".to_string(),
            ))
        }
    }

    fn compile_class(&mut self) -> Result<(), Error> {
        self.writeln("<class>")?;
        self.increment_indent();

        self.writeln("<keyword> class </keyword>")?;

        if let Identifier(identifier) = self.advance()? {
            let msg = format!("<identifier> {} </identifier>", identifier);
            self.writeln(msg.as_str())
        } else {
            Err(Error::UnexpectedInput("not identifier".to_string()))
        }?;

        if &Symbol(Symbol::BraceLeft) == self.advance()? {
            self.writeln("<symbol> { </symbol>")
        } else {
            Err(Error::UnexpectedInput("not {".to_string()))
        }?;

        self.compile_class_var_dec()?;
        self.compile_subroutine_dec()?;

        if &Symbol(Symbol::BraceRight) == self.peek()? {
            self.writeln("<symbol> } </symbol>")
        } else {
            Err(Error::UnexpectedInput("not }".to_string()))
        }?;

        self.decrement_indent();
        self.writeln("</class>")
    }

    fn compile_class_var_dec(&mut self) -> Result<(), Error> {
        match self.peek()? {
            Keyword(ref keyword) if keyword == &Static || keyword == &Field => {
                let msg = format!("<keyword> {} </keyword>", keyword);

                self.writeln("<classVarDec>")?;
                self.increment_indent();

                self.writeln(msg.as_str())?;
            }

            _ => return Ok(()),
        }
        self.current_index += 1;

        // type
        match self.advance()? {
            Keyword(Int) => self.writeln("<keyword> int </keyword>"),
            Keyword(Char) => self.writeln("<keyword> char </keyword>"),
            Keyword(Boolean) => self.writeln("<keyword> boolean </keyword>"),
            Identifier(class_name) => {
                let msg = format!("<identifier> {} </identifier>", class_name);
                self.writeln(msg.as_str())
            }

            _ => Err(Error::UnexpectedInput("not type".to_string())),
        }?;

        // first variable name
        if let Identifier(variable_name) = self.advance()? {
            let msg = format!("<identifier> {} </identifier>", variable_name);
            self.writeln(msg.as_str())
        } else {
            Err(Error::UnexpectedInput("not variable name".to_string()))
        }?;

        loop {
            match self.advance()? {
                Symbol(Symbol::Comma) => self.writeln("<symbol> , </symbol>"),
                Symbol(Symbol::SemiColon) => {
                    self.writeln("<symbol> ; </symbol>")?;
                    break;
                }

                _ => break,
            }?;

            // variable name
            if let Identifier(variable_name) = self.advance()? {
                let msg = format!("<identifier> {} </identifier>", variable_name);
                self.writeln(msg.as_str())
            } else {
                Err(Error::UnexpectedInput("not type".to_string()))
            }?;
        }

        self.decrement_indent();
        self.writeln("</classVarDec>")?;

        self.compile_class_var_dec()
    }

    fn compile_subroutine_dec(&mut self) -> Result<(), Error> {
        match self.peek()? {
            Keyword(keyword)
                if keyword == &Constructor || keyword == &Method || keyword == &Function =>
            {
                let msg = format!("<keyword> {} </keyword>", keyword);

                self.writeln("<subroutineDec>")?;
                self.increment_indent();
                self.writeln(msg.as_str())
            }

            _ => return Ok(()),
        }?;
        self.current_index += 1;

        // type
        match self.advance()? {
            Keyword(Int) => self.writeln("<keyword> int </keyword>"),
            Keyword(Char) => self.writeln("<keyword> char </keyword>"),
            Keyword(Boolean) => self.writeln("<keyword> boolean </keyword>"),
            Keyword(Void) => self.writeln("<keyword> void </keyword>"),
            Identifier(class_name) => {
                let msg = format!("<identifier> {} </identifier>", class_name);
                self.writeln(msg.as_str())
            }

            _ => Err(Error::UnexpectedInput("not type".to_string())),
        }?;

        // name
        match self.advance()? {
            Identifier(name) => {
                let msg = format!("<identifier> {} </identifier>", name);
                self.writeln(msg.as_str())
            }

            _ => Err(Error::UnexpectedInput("not name".to_string())),
        }?;

        if &Symbol(Symbol::ParenthesLeft) == self.advance()? {
            self.writeln("<symbol> ( </symbol>")
        } else {
            Err(Error::UnexpectedInput("not (".to_string()))
        }?;

        self.writeln("<parameterList>")?;
        self.increment_indent();

        self.compile_parameter_list()?;

        self.decrement_indent();
        self.writeln("</parameterList>")?;

        if &Symbol(Symbol::ParenthesRight) == self.advance()? {
            self.writeln("<symbol> ) </symbol>")
        } else {
            Err(Error::UnexpectedInput("not )".to_string()))
        }?;

        // subroutne body.
        self.writeln("<subroutineBody>")?;
        self.increment_indent();

        if &Symbol(Symbol::BraceLeft) == self.advance()? {
            self.writeln("<symbol> { </symbol>")
        } else {
            Err(Error::UnexpectedInput("not {".to_string()))
        }?;

        self.compile_var_dec()?;
        self.compile_statements()?;

        if &Symbol(Symbol::BraceRight) == self.advance()? {
            self.writeln("<symbol> } </symbol>")
        } else {
            Err(Error::UnexpectedInput("not }".to_string()))
        }?;

        self.decrement_indent();
        self.writeln("</subroutineBody>")?;

        self.decrement_indent();
        self.writeln("</subroutineDec>")?;

        self.compile_subroutine_dec()
    }

    fn compile_parameter_list(&mut self) -> Result<(), Error> {
        // type
        match self.peek()? {
            Keyword(Int) => self.writeln("<keyword> int </keyword>"),
            Keyword(Char) => self.writeln("<keyword> char </keyword>"),
            Keyword(Boolean) => self.writeln("<keyword> boolean </keyword>"),
            Keyword(Void) => self.writeln("<keyword> void </keyword>"),
            Identifier(class_name) => {
                let msg = format!("<identifier> {} </identifier>", class_name);
                self.writeln(msg.as_str())
            }

            _ => return Ok(()),
        }?;
        self.current_index += 1;

        // name
        if let Identifier(arg_name) = self.advance()? {
            let msg = format!("<identifier> {} </identifier>", arg_name);
            self.writeln(msg.as_str())
        } else {
            Err(Error::UnexpectedInput("not identifier".to_string()))
        }?;

        if let Symbol(Symbol::Comma) = self.peek()? {
            self.current_index += 1;
            self.writeln("<symbol> , </symbol>")?;
            self.compile_parameter_list()?;
        }

        Ok(())
    }

    fn compile_var_dec(&mut self) -> Result<(), Error> {
        if let Keyword(Var) = self.peek()? {
            self.writeln("<varDec>")?;
            self.increment_indent();
            self.writeln("<keyword> var </keyword>")?;
        } else {
            return Ok(());
        }
        self.current_index += 1;

        // type
        match self.advance()? {
            Keyword(Int) => self.writeln("<keyword> int </keyword>"),
            Keyword(Char) => self.writeln("<keyword> char </keyword>"),
            Keyword(Boolean) => self.writeln("<keyword> boolean </keyword>"),
            Keyword(Void) => self.writeln("<keyword> void </keyword>"),
            Identifier(class_name) => {
                let msg = format!("<identifier> {} </identifier>", class_name);
                self.writeln(msg.as_str())
            }

            _ => return Ok(()),
        }?;

        loop {
            // name
            if let Identifier(arg_name) = self.advance()? {
                let msg = format!("<identifier> {} </identifier>", arg_name);
                self.writeln(msg.as_str())
            } else {
                Err(Error::UnexpectedInput("not identifier".to_string()))
            }?;

            match self.advance()? {
                Symbol(Symbol::Comma) => {
                    self.writeln("<symbol> , </symbol>")?;
                    continue;
                }

                Symbol(Symbol::SemiColon) => {
                    self.writeln("<symbol> ; </symbol>")?;
                    break;
                }

                _ => return Err(Error::UnexpectedInput("not , or ;".to_string())),
            }
        }

        self.decrement_indent();
        self.writeln("</varDec>")?;

        self.compile_var_dec()
    }

    fn compile_statements(&mut self) -> Result<(), Error> {
        self.writeln("<statements>")?;
        self.increment_indent();

        loop {
            match self.peek()? {
                Keyword(Let) => self.compile_let(),
                Keyword(If) => self.compile_if(),
                Keyword(While) => self.compile_while(),
                Keyword(Do) => self.compile_do(),
                Keyword(Return) => self.compile_return(),
                _ => break,
            }?;
        }

        self.decrement_indent();
        self.writeln("</statements>")?;

        Ok(())
    }

    fn compile_let(&mut self) -> Result<(), Error> {
        if let Keyword(Let) = self.advance()? {
            self.writeln("<letStatement>")?;
            self.increment_indent();
            self.writeln("<keyword> let </keyword>")?;
        } else {
            return Err(Error::UnexpectedInput("bug".to_string()));
        }

        // var name.
        if let Identifier(arg_name) = self.advance()? {
            let msg = format!("<identifier> {} </identifier>", arg_name);
            self.writeln(msg.as_str())
        } else {
            Err(Error::UnexpectedInput("not identifier".to_string()))
        }?;

        // array index.
        if let Symbol(Symbol::BracketLeft) = self.peek()? {
            self.current_index += 1;
            self.writeln("<symbol> [ </symbol>")?;

            self.compile_expression()?;

            if let Symbol(Symbol::BracketRight) = self.advance()? {
                self.writeln("<symbol> ] </symbol>")?;
            } else {
                return Err(Error::UnexpectedInput("not ]".to_string()));
            }
        }

        if let Symbol(Symbol::Equal) = self.advance()? {
            self.writeln("<symbol> = </symbol>")?;
        } else {
            return Err(Error::UnexpectedInput("not =".to_string()));
        }

        self.compile_expression()?;

        if let Symbol(Symbol::SemiColon) = self.advance()? {
            self.writeln("<symbol> ; </symbol>")?;
        } else {
            return Err(Error::UnexpectedInput("not ;".to_string()));
        }

        self.decrement_indent();
        self.writeln("</letStatement>")
    }

    fn compile_if(&mut self)-> Result<(), Error> {
        if let Keyword(If) = self.advance()? {
            self.writeln("<ifStatement>")?;
            self.increment_indent();
            self.writeln("<keyword> if </keyword>")
        } else {
            Err(Error::UnexpectedInput("bug".to_string()))
        }?;

        if let Symbol(Symbol::ParenthesLeft) = self.advance()? {
            self.writeln("<symbol> ( </symbol>")
        } else {
            Err(Error::UnexpectedInput("not (".to_string()))
        }?;

        self.compile_expression()?;

        if let Symbol(Symbol::ParenthesRight) = self.advance()? {
            self.writeln("<symbol> ) </symbol>")
        } else {
            Err(Error::UnexpectedInput("not )".to_string()))
        }?;

        if let Symbol(Symbol::BraceLeft) = self.advance()? {
            self.writeln("<symbol> { </symbol>")
        } else {
            Err(Error::UnexpectedInput("not {".to_string()))
        }?;

        self.compile_statements()?;

        if let Symbol(Symbol::BraceRight) = self.advance()? {
            self.writeln("<symbol> } </symbol>")
        } else {
            Err(Error::UnexpectedInput("not }".to_string()))
        }?;

        if let Keyword(Else) = self.peek()? {
            self.current_index += 1;
            self.writeln("<keyword> else </keyword>")?;

            if let Symbol(Symbol::BraceLeft) = self.advance()? {
                self.writeln("<symbol> { </symbol>")
            } else {
                Err(Error::UnexpectedInput("not {".to_string()))
            }?;

            self.compile_statements()?;

            if let Symbol(Symbol::BraceRight) = self.advance()? {
                self.writeln("<symbol> } </symbol>")
            } else {
                Err(Error::UnexpectedInput("not }".to_string()))
            }?;
        }

        self.decrement_indent();
        self.writeln("</ifStatement>")
    }

    fn compile_while(&mut self)-> Result<(), Error> {
        if let Keyword(While) = self.advance()? {
            self.writeln("<whileStatement>")?;
            self.increment_indent();
            self.writeln("<keyword> while </keyword>")
        } else {
            Err(Error::UnexpectedInput("bug".to_string()))
        }?;

        if let Symbol(Symbol::ParenthesLeft) = self.advance()? {
            self.writeln("<symbol> ( </symbol>")
        } else {
            Err(Error::UnexpectedInput("not (".to_string()))
        }?;

        self.compile_expression()?;

        if let Symbol(Symbol::ParenthesRight) = self.advance()? {
            self.writeln("<symbol> ) </symbol>")
        } else {
            Err(Error::UnexpectedInput("not )".to_string()))
        }?;

        if let Symbol(Symbol::BraceLeft) = self.advance()? {
            self.writeln("<symbol> { </symbol>")
        } else {
            Err(Error::UnexpectedInput("not {".to_string()))
        }?;

        self.compile_statements()?;

        if let Symbol(Symbol::BraceRight) = self.advance()? {
            self.writeln("<symbol> } </symbol>")
        } else {
            Err(Error::UnexpectedInput("not }".to_string()))
        }?;

        self.decrement_indent();
        self.writeln("</whileStatement>")
    }

    fn compile_do(&mut self) -> Result<(), Error> {
        if let Keyword(Do) = self.advance()? {
            self.writeln("<doStatement>")?;
            self.increment_indent();
            self.writeln("<keyword> do </keyword>")?;
        } else {
            return Err(Error::UnexpectedInput("bug".to_string()));
        }

        self.compile_subroutine_call()?;

        if let Symbol(Symbol::SemiColon) = self.advance()? {
            self.writeln("<symbol> ; </symbol>")?;
        } else {
            return Err(Error::UnexpectedInput("not ;".to_string()));
        }

        self.decrement_indent();
        self.writeln("</doStatement>")
    }

    fn compile_return(&mut self) -> Result<(), Error> {
        if let Keyword(Return) = self.advance()? {
            self.writeln("<returnStatement>")?;
            self.increment_indent();
            self.writeln("<keyword> return </keyword>")
        } else {
            Err(Error::UnexpectedInput("bug".to_string()))
        }?;

        if let Symbol(Symbol::SemiColon) = self.peek()? {
            self.current_index += 1;
            self.writeln("<symbol> ; </symbol>")?;
        } else {
            self.compile_expression()?;
            if let Symbol(Symbol::SemiColon) = self.advance()? {
                self.writeln("<symbol> ; </symbol>")
            } else {
                Err(Error::UnexpectedInput("not ;".to_string()))
            }?;
        }

        self.decrement_indent();
        self.writeln("</returnStatement>")
    }

    fn compile_expression(&mut self) -> Result<(), Error> {
        self.writeln("<expression>")?;
        self.increment_indent();

        self.compile_term()?;

        match self.peek()? {
            Symbol(symbol)
                if symbol == &Symbol::Plus
                    || symbol == &Symbol::Minus
                    || symbol == &Symbol::Star
                    || symbol == &Symbol::Slash
                    || symbol == &Symbol::And
                    || symbol == &Symbol::Or
                    || symbol == &Symbol::Lt
                    || symbol == &Symbol::Gt
                    || symbol == &Symbol::Equal =>
            {
                let msg = format!("<symbol> {} </symbol>", symbol);
                self.writeln(msg.as_str())?;

                self.current_index += 1;
                self.compile_term()
            }

            _ => Ok(()),
        }?;

        self.decrement_indent();
        self.writeln("</expression>")
    }

    fn compile_term(&mut self) -> Result<(), Error> {
        self.writeln("<term>")?;
        self.increment_indent();

        // TODO: refine
        let token = self.advance()?.clone();
        match token {
            IntegerConstant(value) => {
                let msg = format!("<integerConstant> {} </integerConstant>", value);
                self.writeln(msg.as_str())
            }

            StringConstant(value) => {
                let msg = format!("<stringConstant> {} </stringConstant>", value);
                self.writeln(msg.as_str())
            }

            Keyword(True) => self.writeln("<keyword> true </keyword>"),

            Keyword(False) => self.writeln("<keyword> false </keyword>"),

            Keyword(Null) => self.writeln("<keyword> null </keyword>"),

            Keyword(This) => self.writeln("<keyword> this </keyword>"),

            Identifier(var_name) => {
                let token = self.peek()?.clone();
                match token {
                    Symbol(Symbol::ParenthesLeft) => {
                        self.current_index -= 1;
                        self.compile_subroutine_call()
                    }

                    Symbol(Symbol::Dot) => {
                        self.current_index -= 1;
                        self.compile_subroutine_call()
                    }

                    _ => {
                        // var_name
                        let msg = format!("<identifier> {} </identifier>", var_name);
                        self.writeln(msg.as_str())?;

                        match self.peek()? {
                            Symbol(Symbol::BracketLeft) => {
                                // array index.
                                self.current_index += 1;
                                self.writeln("<symbol> [ </symbol>")?;

                                self.compile_expression()?;

                                if let Symbol(Symbol::BracketRight) = self.advance()? {
                                    self.writeln("<symbol> ] </symbol>")
                                } else {
                                    Err(Error::UnexpectedInput("not ]".to_string()))
                                }
                            }

                            _ => Ok(()),
                        }
                    }
                }
            }

            Symbol(Symbol::ParenthesLeft) => {
                self.writeln("<symbol> ( </symbol>")?;
                self.compile_expression()?;

                if let Symbol(Symbol::ParenthesRight) = self.advance()? {
                    self.writeln("<symbol> ) </symbol>")
                } else {
                    Err(Error::UnexpectedInput("not )".to_string()))
                }
            }

            Symbol(Symbol::Minus) => {
                self.writeln("<symbol> - </symbol>")?;
                self.compile_term()
            },

            Symbol(Symbol::Not) => {
                self.writeln("<symbol> ~ </symbol>")?;
                self.compile_term()
            },

            _ => Err(Error::UnexpectedInput("not expression".to_string())),
        }?;

        self.decrement_indent();
        self.writeln("</term>")
    }

    fn compile_subroutine_call(&mut self) -> Result<(), Error> {
        match self.tokens[self.current_index + 1] {
            Symbol(Symbol::ParenthesLeft) => {
                if let Identifier(subroutine_name) = self.advance()? {
                    let msg = format!("<identifier> {} </identifier>", subroutine_name);
                    self.writeln(msg.as_str())
                } else {
                    Err(Error::UnexpectedInput("not (".to_string()))
                }?;

                self.advance()?;
                self.writeln("<symbol> ( </symbol>")?;

                if let Symbol(Symbol::ParenthesRight) = self.peek()? {
                    self.current_index += 1;
                    self.writeln("<expressionList>")?;
                    self.writeln("</expressionList>")?;
                    self.writeln("<symbol> ) </symbol>")
                } else {
                    self.compile_expression_list()?;
                    if let Symbol(Symbol::ParenthesRight) = self.advance()? {
                        self.writeln("<symbol> ) </symbol>")
                    } else {
                        Err(Error::UnexpectedInput("not )".to_string()))
                    }
                }?;

                Ok(())
            }

            Symbol(Symbol::Dot) => {
                if let Identifier(name) = self.advance()? {
                    let msg = format!("<identifier> {} </identifier>", name);
                    self.writeln(msg.as_str())
                } else {
                    Err(Error::UnexpectedInput("not identifier".to_string()))
                }?;

                self.advance()?;
                self.writeln("<symbol> . </symbol>")?;

                if let Identifier(subroutine_name) = self.advance()? {
                    let msg = format!("<identifier> {} </identifier>", subroutine_name);
                    self.writeln(msg.as_str())
                } else {
                    Err(Error::UnexpectedInput("not identifier".to_string()))
                }?;

                if let Symbol(Symbol::ParenthesLeft) = self.advance()? {
                    self.writeln("<symbol> ( </symbol>")
                } else {
                    Err(Error::UnexpectedInput("not (".to_string()))
                }?;

                if let Symbol(Symbol::ParenthesRight) = self.peek()? {
                    self.current_index += 1;

                    self.writeln("<expressionList>")?;
                    self.writeln("</expressionList>")?;
                    self.writeln("<symbol> ) </symbol>")
                } else {
                    self.compile_expression_list()?;
                    if let Symbol(Symbol::ParenthesRight) = self.advance()? {
                        self.writeln("<symbol> ) </symbol>")
                    } else {
                        Err(Error::UnexpectedInput("not )".to_string()))
                    }
                }?;

                Ok(())
            }

            _ => Err(Error::UnexpectedInput("not subroutine call".to_string())),
        }
    }

    fn compile_expression_list(&mut self) -> Result<(), Error> {
        self.writeln("<expressionList>")?;
        self.increment_indent();

        self.compile_expression()?;

        while let Symbol(Symbol::Comma) = self.peek()? {
            self.current_index += 1;
            self.writeln("<symbol> , </symbol>")?;
            self.compile_expression()?;
        }

        self.decrement_indent();
        self.writeln("</expressionList>")
    }

    fn writeln(&mut self, msg: &str) -> Result<(), Error> {
        self.writer
            .write_all(self.spaces.as_bytes())
            .map_err(Error::IoError)?;
        self.writer
            .write_all(msg.as_bytes())
            .map_err(Error::IoError)?;
        self.writer.write_all(b"\n").map_err(Error::IoError)
    }

    fn increment_indent(&mut self) {
        for _ in 0..INDENT_SIZE {
            self.spaces.push_str(" ");
        }
    }

    fn decrement_indent(&mut self) {
        debug_assert!(0 <= (self.spaces.len() - INDENT_SIZE));

        self.spaces.truncate(self.spaces.len() - INDENT_SIZE);
    }

    fn advance(&mut self) -> Result<&Token, Error> {
        if self.has_more_token() {
            let token = &self.tokens[self.current_index];

            self.current_index += 1;

            Ok(token)
        } else {
            Err(Error::BrokenInput)
        }
    }

    fn peek(&self) -> Result<&Token, Error> {
        if self.has_more_token() {
            Ok(&self.tokens[self.current_index])
        } else {
            Err(Error::BrokenInput)
        }
    }

    fn has_more_token(&self) -> bool {
        self.current_index < self.tokens.len()
    }
}

#[derive(Debug)]
pub enum Error {
    TokenizerError(tokenizer::Error),
    UnexpectedInput(String),
    // TODO: Found xxx, you should put yyyy.
    MustBe(String, String),
    IoError(io::Error),
    BrokenInput,
}

impl From<tokenizer::Error> for Error {
    fn from(error: tokenizer::Error) -> Error {
        Error::TokenizerError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn test_parser() {
        let src = "class Main {\n\
                     static int x, y;\n\
                     field Hoge a;\n\
                     function void main() {\n\
                       var int i, j;\n\
                       let i = 100;\n\
                       do Output.printString(\"THE AVERAGE IS: \");\n\
                       return;\n\
                     }\n\
                   }\n\
                   ";
        let mut tokenizer = Tokenizer::new(src);
        let mut tokens = Vec::new();

        loop {
            match tokenizer.advance() {
                Ok(Some(token)) => tokens.push(token.clone()),
                Ok(None) => break,

                Err(error) => panic!("tokenize error: {:?}", error),
            }
        }

        println!("{:?}", tokens);
        let mut buf = Vec::<u8>::new();
        let mut parser = Parser::new(tokens, &mut buf);

        let r = parser.compile();
        println!("{:?}", r);
        drop(parser);

        println!("{}", std::str::from_utf8(buf.as_slice()).unwrap());
        assert_eq!(true, r.is_ok());

        let expected = concat!(
            "<class>\n",
            "  <keyword> class </keyword>\n",
            "  <identifier> Main </identifier>\n",
            "</class>\n"
        );
        assert_eq!(expected, std::str::from_utf8(buf.as_slice()).unwrap());
    }
}
