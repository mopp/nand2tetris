mod code_writer;
mod parser;

use code_writer::CodeWriter;
use parser::{CommandType, Parser};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

fn main() -> Result<(), std::io::Error> {
    let src_path = env::args()
        .nth(1)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "No argument"))
        .map(PathBuf::from)?;

    // TODO: Support multiple files.
    let mut src = File::open(src_path.as_path()).map(BufReader::new)?;
    let mut dst = File::create(src_path.with_extension("asm"))?;

    translate(&mut src, src_path.as_path(), &mut dst)
}

// TODO: Support multiple files.
fn translate<R: BufRead + Seek, W: Write>(
    src: &mut R,
    path: &Path,
    dst: &mut W,
) -> Result<(), std::io::Error> {
    let mut writer = CodeWriter::new(dst);

    writer.set_filename(path.file_name().unwrap().to_str().unwrap());
    let mut parser = Parser::new(src)?;
    while parser.has_more_commands() {
        match parser.command_type() {
            CommandType::Push => {
                writer.write_push_pop("push", parser.arg1().as_str(), parser.arg2())
            }
            CommandType::Pop => {
                writer.write_push_pop("pop", parser.arg1().as_str(), parser.arg2())
            }
            _ => unimplemented!(""),
        }

        parser.advance();
    }

    Ok(())
}
