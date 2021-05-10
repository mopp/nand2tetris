mod parser;
mod symbol_table;
mod tokenizer;

use parser::Parser;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::path::PathBuf;
use tokenizer::Tokenizer;

fn main() -> Result<(), io::Error> {
    let given_path = env::args()
        .nth(1)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "No argument"))
        .map(PathBuf::from)?;

    let file_tuples = find_jack_paths(&given_path)?;

    for (jack_path, vm_path) in file_tuples {
        println!("input file: {}", jack_path.to_str().unwrap());
        println!("output file: {}", vm_path.to_str().unwrap());

        let mut jack_file = File::open(&jack_path)?;
        let mut jack_code = String::new();
        jack_file.read_to_string(&mut jack_code)?;

        let mut tokenizer = Tokenizer::new(jack_code.as_str());
        let mut tokens = Vec::new();

        loop {
            match tokenizer.advance() {
                Ok(Some(token)) => {
                    tokens.push(token.clone());
                }

                Ok(None) => break,

                Err(error) => panic!("tokenize error: {:?}", error),
            }
        }

        let mut vm_file = File::create(vm_path)?;
        let mut parser = Parser::new(tokens, &mut vm_file);

        if let Err(error) = parser.compile() {
            panic!("{:?}", error);
        }
    }

    Ok(())
}

// (jack path, vm path)
fn find_jack_paths(path: &Path) -> Result<Vec<(PathBuf, PathBuf)>, Error> {
    if path.is_dir() {
        // Find *.jack files in the given directory.
        path.read_dir()?
            .map(|entry| entry.map(|e| e.path()))
            .filter(|result_path| match result_path {
                Ok(ref path) => match path.extension() {
                    Some(ext) => ext == "jack",
                    _ => false,
                },
                _ => false,
            })
            .collect::<Result<Vec<_>, Error>>()
            .map(|jack_paths| {
                jack_paths
                    .iter()
                    .map(|jack_path| (jack_path.clone(), jack_path.with_extension("vm")))
                    .collect()
            })
    } else {
        // Check the given file is vm file or not.
        match path.extension() {
            Some(ext) if ext == "jack" => Ok(vec![(path.to_path_buf(), path.with_extension("vm"))]),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "The given file is NOT vm file",
            )),
        }
    }
}
