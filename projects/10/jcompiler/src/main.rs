mod tokenizer;

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use tokenizer::Tokenizer;

fn main() -> Result<(), std::io::Error> {
    let given_path = env::args()
        .nth(1)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "No argument"))
        .map(PathBuf::from)?;

    let file_tuples = find_jack_paths(&given_path)?;

    for (jack_path, _token_path, _parse_path) in file_tuples {
        let mut jack_file = File::open(&jack_path)?;
        let mut jack_code = String::new();
        jack_file.read_to_string(&mut jack_code)?;

        let mut tokenizer = Tokenizer::new(jack_code.as_str());
        while let Ok(Some(token)) = tokenizer.next() {
            println!("{:?}", token);
        }
    }

    Ok(())
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
