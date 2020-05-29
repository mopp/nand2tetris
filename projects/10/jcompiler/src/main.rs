mod tokenizer;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use tokenizer::Token;
use tokenizer::Tokenizer;

fn main() -> Result<(), io::Error> {
    let given_path = env::args()
        .nth(1)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "No argument"))
        .map(PathBuf::from)?;

    let file_tuples = find_jack_paths(&given_path)?;

    for (jack_path, token_path, parse_path) in file_tuples {
        println!("input file: {}", jack_path.to_str().unwrap());
        println!("token file: {}", token_path.to_str().unwrap());
        println!("output file: {}", parse_path.to_str().unwrap());

        let mut jack_file = File::open(&jack_path)?;
        let mut jack_code = String::new();
        jack_file.read_to_string(&mut jack_code)?;

        let mut token_file = File::create(token_path)?;
        token_file.write_all(b"<tokens>\n")?;

        let mut tokenizer = Tokenizer::new(jack_code.as_str());
        loop {
            match tokenizer.next() {
                // Finish reading the all tokens.
                Ok(None) => break,
                Ok(Some(token)) => write_token_to_file(&mut token_file, token)?,
                Err(error) => panic!("tokenize error: {:?}", error),
            }
        }

        token_file.write_all(b"</tokens>\n")?;
    }

    Ok(())
}

fn write_token_to_file(file: &mut File, token: Token) -> Result<(), io::Error> {
    let token_str = match token {
        Token::Keyword(keyword) => {
            use tokenizer::Keyword::*;
            let k = match keyword {
                Class => "class",
                Constructor => "constructor",
                Function => "function",
                Method => "method",
                Field => "field",
                Static => "static",
                Var => "var",
                Int => "int",
                Char => "char",
                Boolean => "boolean",
                Void => "void",
                True => "true",
                False => "false",
                Null => "null",
                This => "this",
                Let => "let",
                Do => "do",
                If => "if",
                Else => "else",
                While => "while",
                Return => "return",
            };

            format!("<keyword> {} </keyword>\n", k)
        }

        Token::Symbol(symbol) => {
            use tokenizer::Symbol::*;
            let s = match symbol {
                ParenthesLeft => "(",
                ParenthesRight => ")",
                BracketLeft => "[",
                BracketRight => "]",
                BraceLeft => "{",
                BraceRight => "}",
                Dot => ".",
                Comma => ",",
                SemiColon => ";",
                Plus => "+",
                Minus => "-",
                Star => "*",
                Slash => "/",
                And => "&",
                Or => "|",
                Lt => "&lt;",
                Gt => "&gt;",
                Equal => "=",
                Not => "!",
            };
            format!("<symbol> {} </symbol>\n", s)
        }

        Token::Identifier(identifier) => format!("<identifier> {} </identifier>\n", identifier),

        Token::IntegerConstant(integer) => {
            format!("<integerConstant> {} </integerConstant>\n", integer)
        }

        Token::StringConstant(string) => format!("<stringConstant> {} </stringConstant>\n", string),
    };

    file.write_all(token_str.as_bytes())
}

// (jack path, token path, parse path)
fn find_jack_paths(path: &PathBuf) -> Result<Vec<(PathBuf, PathBuf, PathBuf)>, Error> {
    if path.is_dir() {
        // Find *.jack files in the given directory.
        path.read_dir()?
            .map(|entry| entry.map(|e| e.path()))
            .filter(|result_path| {
                if let Ok(ref path) = result_path {
                    match path.extension() {
                        Some(ext) if ext == "jack" => true,
                        Some(_) => false,
                        _ => true,
                    }
                } else {
                    true
                }
            })
            .collect::<Result<Vec<_>, Error>>()
            .and_then(|jack_paths| {
                Ok(jack_paths
                    .iter()
                    .map(|jack_path| {
                        let mut token_path = jack_path.clone();
                        token_path.set_file_name(format!(
                            "{}T.xml",
                            jack_path.file_stem().unwrap().to_str().unwrap()
                        ));

                        (
                            jack_path.clone(),
                            token_path,
                            jack_path.with_extension("xml"),
                        )
                    })
                    .collect())
            })
    } else {
        // Check the given file is vm file or not.
        match path.extension() {
            Some(ext) if ext == "jack" => {
                let mut token_path = path.clone();
                token_path.set_file_name(format!(
                    "{}T.xml",
                    path.file_stem().unwrap().to_str().unwrap()
                ));

                Ok(vec![(path.clone(), token_path, path.with_extension("xml"))])
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "The given file is NOT vm file",
            )),
        }
    }
}
