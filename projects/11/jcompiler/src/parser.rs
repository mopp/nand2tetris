use crate::symbol_table::{Kind, SymbolTable};
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
    symbol_table: SymbolTable,
}

impl<W: Write> Parser<W> {
    pub fn new(tokens: Vec<Token>, writer: W) -> Self {
        Self {
            tokens,
            current_index: 0,
            spaces: String::new(),
            writer: BufWriter::new(writer),
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn compile(&mut self) -> Result<(), Error> {
        self.compile_class()
    }

    fn compile_class(&mut self) -> Result<(), Error> {
        if &Keyword(Class) != self.advance()? {
            return Err(Error::UnexpectedInput(
                "top level component have to be class".to_string(),
            ));
        }

        let class_name = if let Identifier(identifier) = self.advance()? {
            identifier.clone()
        } else {
            return Err(Error::UnexpectedInput("not identifier".to_string()));
        };

        self.compile_brace_block(Box::new(move |this| {
            this.symbol_table = SymbolTable::new();
            this.compile_class_var_dec()?;
            this.compile_subroutine_dec(class_name)
        }))
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

        let kind = match self.advance()? {
            Keyword(Static) => Kind::Static,
            Keyword(Field) => Kind::Field,
            _ => panic!("unexpected!"),
        };

        // type
        let itype = match self.advance()?.clone() {
            Keyword(Int) => {
                self.writeln("<keyword> int </keyword>")?;

                "int".to_string()
            }
            Keyword(Char) => {
                self.writeln("<keyword> char </keyword>")?;

                "char".to_string()
            }
            Keyword(Boolean) => {
                self.writeln("<keyword> boolean </keyword>")?;

                "boolean".to_string()
            }
            Identifier(class_name) => {
                let msg = format!("<identifier> {} </identifier>", class_name);
                self.writeln(msg.as_str())?;

                class_name.clone()
            }

            _ => return Err(Error::UnexpectedInput("not type".to_string())),
        };

        // first variable name
        if let Identifier(variable_name) = self.advance()?.clone() {
            self.symbol_table
                .define(variable_name.to_string(), itype.to_string(), kind);

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
            if let Identifier(variable_name) = self.advance()?.clone() {
                self.symbol_table
                    .define(variable_name.to_string(), itype.to_string(), kind);

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

    fn compile_subroutine_dec(&mut self, class_name: String) -> Result<(), Error> {
        self.symbol_table.start_subroutine();

        let count_args = match self.peek()? {
            Keyword(Constructor) => 0,
            Keyword(Function) => 0,
            Keyword(Method) => 1,
            _ => return Ok(()),
        };
        self.current_index += 1;

        // Discard the type.
        match self.advance()? {
            Keyword(Int) => (),
            Keyword(Char) => (),
            Keyword(Boolean) => (),
            Keyword(Void) => (),
            Identifier(_) => (),
            _ => return Err(Error::UnexpectedInput("not type".to_string())),
        }

        let subroutine_name = if let Identifier(name) = self.advance()? {
            name.clone()
        } else {
            return Err(Error::UnexpectedInput("not name".to_string()));
        };

        // Compile parameter list.
        {
            let class_name = class_name.clone();
            self.compile_paren_block(Box::new(move |this| {
                let count_args = count_args + this.compile_parameter_list()?;
                let msg = format!("function {}.{} {}", class_name, subroutine_name, count_args);
                this.writeln(msg.as_str())
            }))?;
        }

        // Compile subrountine body.
        self.compile_brace_block(Box::new(move |this| {
            this.compile_var_dec()?;
            this.compile_statements()
        }))?;

        self.compile_subroutine_dec(class_name)
    }

    fn compile_parameter_list(&mut self) -> Result<usize, Error> {
        // type
        let itype = match self.peek()?.clone() {
            Keyword(Int) => {
                self.writeln("<keyword> int </keyword>")?;

                "int".to_string()
            }
            Keyword(Char) => {
                self.writeln("<keyword> char </keyword>")?;

                "char".to_string()
            }
            Keyword(Boolean) => {
                self.writeln("<keyword> boolean </keyword>")?;

                "boolean".to_string()
            }
            Identifier(class_name) => {
                let msg = format!("<identifier> {} </identifier>", class_name);
                self.writeln(msg.as_str())?;

                class_name.clone()
            }

            _ => return Ok(0),
        };
        self.current_index += 1;

        // name
        if let Identifier(arg_name) = self.advance()?.clone() {
            self.symbol_table
                .define(arg_name.to_string(), itype.to_string(), Kind::Arg);
            let msg = format!("<identifier> {} </identifier>", arg_name);
            self.writeln(msg.as_str())
        } else {
            Err(Error::UnexpectedInput("not identifier".to_string()))
        }?;

        if let Symbol(Symbol::Comma) = self.peek()? {
            self.current_index += 1;
            self.writeln("<symbol> , </symbol>")?;
            return Ok(self.compile_parameter_list()? + 1);
        }

        Ok(1)
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
        let itype = match self.advance()?.clone() {
            Keyword(Int) => {
                self.writeln("<keyword> int </keyword>")?;

                "int".to_string()
            }
            Keyword(Char) => {
                self.writeln("<keyword> char </keyword>")?;

                "char".to_string()
            }
            Keyword(Boolean) => {
                self.writeln("<keyword> boolean </keyword>")?;

                "boolean".to_string()
            }
            Identifier(class_name) => {
                let msg = format!("<identifier> {} </identifier>", class_name);
                self.writeln(msg.as_str())?;

                class_name.clone()
            }

            _ => return Err(Error::UnexpectedInput("not type".to_string())),
        };

        loop {
            // name
            if let Identifier(var_name) = self.advance()?.clone() {
                self.symbol_table
                    .define(var_name.to_string(), itype.to_string(), Kind::Var);
                let msg = format!("<identifier> {} </identifier>", var_name);
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

    fn compile_if(&mut self) -> Result<(), Error> {
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

    fn compile_while(&mut self) -> Result<(), Error> {
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
            }

            Symbol(Symbol::Not) => {
                self.writeln("<symbol> ~ </symbol>")?;
                self.compile_term()
            }

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

    fn compile_brace_block(
        &mut self,
        f: Box<dyn FnOnce(&mut Self) -> Result<(), Error>>,
    ) -> Result<(), Error> {
        self.compile_block((Symbol::BraceLeft, Symbol::BraceRight), f)
    }

    fn compile_paren_block(
        &mut self,
        f: Box<dyn FnOnce(&mut Self) -> Result<(), Error>>,
    ) -> Result<(), Error> {
        self.compile_block((Symbol::ParenthesLeft, Symbol::ParenthesRight), f)
    }

    fn compile_block(
        &mut self,
        surround: (Symbol, Symbol),
        f: Box<dyn FnOnce(&mut Self) -> Result<(), Error>>,
    ) -> Result<(), Error> {
        if &Symbol(surround.0) != self.advance()? {
            return Err(Error::UnexpectedInput("not {".to_string()));
        }

        f(self)?;

        if &Symbol(surround.1) != self.advance()? {
            return Err(Error::UnexpectedInput("not }".to_string()));
        }

        Ok(())
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
            self.spaces.push(' ');
        }
    }

    fn decrement_indent(&mut self) {
        debug_assert!(INDENT_SIZE <= self.spaces.len());

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
        assert!(r.is_ok());

        // let expected = concat!(
        //     "<class>\n",
        //     "  <keyword> class </keyword>\n",
        //     "  <identifier> Main </identifier>\n",
        //     "</class>\n"
        // );
        // assert_eq!(expected, std::str::from_utf8(buf.as_slice()).unwrap());
    }
}
