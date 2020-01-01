mod code;
mod parser;
mod symbol_table;

use parser::{CommandType, Parser};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use symbol_table::SymbolTable;

static VARIABLE_ADDRESS_BEGIN: u16 = 16;

fn main() -> Result<(), std::io::Error> {
    let src_path = env::args()
        .nth(1)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "No argument"))
        .map(PathBuf::from)?;

    let mut src = File::open(src_path.as_path()).map(BufReader::new)?;
    let mut dst = File::create(src_path.with_extension("hack"))?;
    assemble(&mut src, &mut dst)
}

fn assemble<R: BufRead + Seek, W: Write>(src: &mut R, dst: &mut W) -> Result<(), std::io::Error> {
    let mut symbol_table = SymbolTable::new();

    pass1(src, &mut symbol_table)?;

    src.seek(SeekFrom::Start(0))?;

    pass2(src, dst, &mut symbol_table)
}

/// Read the all lines in order to create symbol table.
fn pass1<R: BufRead>(src: &mut R, symbol_table: &mut SymbolTable) -> Result<(), std::io::Error> {
    let mut current_address = 0;
    let mut parser = Parser::new(src);

    while parser.has_more_commands() {
        if parser.command_type() == CommandType::Label {
            // Record the label.
            symbol_table.add_entry(parser.symbol(), current_address);
        } else {
            current_address += 1;
        }

        parser.advance();
    }

    Ok(())
}

/// Generate codes.
fn pass2<R: BufRead, W: Write>(
    src: &mut R,
    dst: &mut W,
    symbol_table: &mut SymbolTable,
) -> Result<(), std::io::Error> {
    let mut parser = Parser::new(src);
    let mut var_address = VARIABLE_ADDRESS_BEGIN;

    while parser.has_more_commands() {
        match parser.command_type() {
            CommandType::Address => {
                let symbol = parser.symbol();
                let n = if let Ok(n) = symbol.parse::<u16>() {
                    // Constant.
                    n
                } else if let Some(n) = symbol_table.get_address(&symbol) {
                    // Use existing variable or label.
                    n
                } else {
                    // Allocate new variable.
                    symbol_table.add_entry(symbol, var_address);

                    let n = var_address;
                    var_address += 1;
                    n
                };

                dst.write_all(format!("0{:015b}\n", n).as_bytes())?;
            }
            CommandType::Compute => {
                let comp = code::comp(parser.comp());
                let dest = code::dest(parser.dest());
                let jump = code::jump(parser.jump());
                let _ = dst.write(format!("111{:}{:}{:}\n", comp, dest, jump).as_bytes())?;
            }
            CommandType::Label => {
                // Nothing to do.
            }
        }

        parser.advance();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::str;

    #[test]
    fn assemble_test() {
        let input: &[u8] = b"@2\n\
                             D=A\n\
                             @3\n\
                             D=D+A\n\
                             @0\n\
                             M=D";
        let mut input = Cursor::new(input);
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

    #[test]
    fn assemble_with_label_test() {
        let input: &[u8] = b"@R0\n\
                             D=M              // D = first number\n\
                             @R1\n\
                             D=D-M            // D = first number - second number\n\
                             @OUTPUT_FIRST\n\
                             D;JGT            // if D>0 (first is greater) goto output_first\n\
                             @R1\n\
                             D=M              // D = second number\n\
                             @OUTPUT_D\n\
                             0;JMP            // goto output_d\n\
                          (OUTPUT_FIRST)\n\
                             @R0             \n\
                             D=M              // D = first number\n\
                          (OUTPUT_D)\n\
                             @R2\n\
                             M=D              // M[2] = D (greatest number)\n\
                          (INFINITE_LOOP)\n\
                             @INFINITE_LOOP\n\
                             0;JMP            // infinite loop";

        let mut input = Cursor::new(input);
        let mut output = Vec::<u8>::new();

        assert_eq!(true, assemble(&mut input, &mut output).is_ok());
        assert_eq!(
            "0000000000000000\n\
             1111110000010000\n\
             0000000000000001\n\
             1111010011010000\n\
             0000000000001010\n\
             1110001100000001\n\
             0000000000000001\n\
             1111110000010000\n\
             0000000000001100\n\
             1110101010000111\n\
             0000000000000000\n\
             1111110000010000\n\
             0000000000000010\n\
             1110001100001000\n\
             0000000000001110\n\
             1110101010000111\n",
            str::from_utf8(&output).unwrap()
        );
    }
}
