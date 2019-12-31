mod code;
mod parser;

use parser::{CommandType, Parser};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

fn main() -> Result<(), std::io::Error> {
    let src_path = env::args()
        .nth(1)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "No argument"))
        .map(PathBuf::from)?;

    let mut src = File::open(src_path.as_path()).map(BufReader::new)?;
    let mut dst = File::create(src_path.with_extension("hack"))?;
    assemble(&mut src, &mut dst)
}

fn assemble<R: BufRead, W: Write>(src: &mut R, dst: &mut W) -> Result<(), std::io::Error> {
    let mut parser = Parser::new(src);

    while parser.has_more_commands() {
        match parser.command_type() {
            CommandType::Address => {
                let _ = parser
                    .symbol()
                    .parse::<u16>()
                    .map(|n| format!("0{:015b}\n", n))
                    .map_err(|e| Error::new(ErrorKind::InvalidInput, e))
                    .and_then(|n| dst.write(n.as_bytes()))?;
            }
            CommandType::Compute => {
                let comp = code::comp(parser.comp());
                let dest = code::dest(parser.dest());
                let jump = code::jump(parser.jump());
                let _ = dst.write(format!("111{:}{:}{:}\n", comp, dest, jump).as_bytes())?;
            }
            CommandType::Label => {
                unimplemented!("TODO");
            }
        }

        parser.advance();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn assemble_test() {
        let mut input: &[u8] = b"@2\n\
                                 D=A\n\
                                 @3\n\
                                 D=D+A\n\
                                 @0\n\
                                 M=D";
        let mut output = Vec::<u8>::new();

        assert_eq!(true, assemble(&mut input, &mut output).is_ok());
        assert_eq!(
            "0000000000000010\n\
             1110110000010000\n\
             0000000000000011\n\
             1110000010010000\n\
             0000000000000000\n\
             1110001100001000\n",
            str::from_utf8(&output).unwrap()
        );
    }
}
