mod code_writer;
mod parser;

use code_writer::CodeWriter;
use parser::Parser;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

fn main() -> Result<(), std::io::Error> {
    let given_path = env::args()
        .nth(1)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "No argument"))
        .map(PathBuf::from)?;

    let (asm_file, vm_paths) = find_vm_paths(&given_path)?;

    println!("input vm files:");
    for vm_path in vm_paths.iter() {
        println!("  {}", vm_path.to_str().unwrap());
    }
    println!("output asm file:");
    println!("  {}", asm_file.to_str().unwrap());

    let mut dst = File::create(asm_file)?;

    let mut writer = CodeWriter::new(&mut dst);

    writer.write_bootstrap_code()?;

    for vm_path in vm_paths {
        let file_name = vm_path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or(Error::new(ErrorKind::Other, "unexpected"))?;

        let mut src = BufReader::new(File::open(&vm_path)?);
        for command in Parser::new(&mut src) {
            writer.put(file_name, &command)?;
        }
    }

    Ok(())
}

fn find_vm_paths(path: &PathBuf) -> Result<(PathBuf, Vec<PathBuf>), Error> {
    if path.is_dir() {
        // Find *.vm files in the given directory.
        path.read_dir()?
            .map(|entry| entry.map(|e| e.path()))
            .filter(|result_path| {
                if let Ok(ref path) = result_path {
                    match path.extension() {
                        Some(ext) if ext == "vm" => true,
                        Some(_) => false,
                        _ => true,
                    }
                } else {
                    true
                }
            })
            .collect::<Result<Vec<_>, Error>>()
            .and_then(|vm_files| {
                let dir_name = path
                    .file_name()
                    .ok_or(Error::new(ErrorKind::InvalidInput, "unexpected"))?;
                let mut path = path.clone();
                path.push(dir_name);
                Ok((path.with_extension("asm"), vm_files))
            })
    } else {
        // Check the given file is vm file or not.
        match path.extension() {
            Some(ext) if ext == "vm" => Ok((path.with_extension("asm"), vec![path.clone()])),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "The given file is NOT vm file",
            )),
        }
    }
}
