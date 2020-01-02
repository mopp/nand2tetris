use std::io::prelude::*;

pub struct CodeWriter<'a, W: Write> {
    target: &'a mut W,
    filename: Option<&'a str>,
}

impl<'a, W: Write> CodeWriter<'a, W> {
    pub fn new(target: &'a mut W) -> Self {
        Self {
            target,
            filename: None,
        }
    }

    pub fn set_filename(&mut self, filename: &'a str) {
        self.filename = Some(filename);
    }

    pub fn write_arithmetic(&mut self, command: &str) -> Result<(), std::io::Error> {
        let instructions = match command {
            "add" => {
                "@SP\n\
                 A=M\n\
                 D=M\n\
                 A=A-1\n\
                 M=D+M\n\
                 D=A\n\
                 @SP\n\
                 M=D"
            }
            _ => "",
        };

        self.target.write_all(instructions.as_bytes())
    }

    pub fn write_push_pop(&mut self, command: &str, segment: &str, index: u16) {
        let instructions = match (command, segment) {
            ("push", "constant") => format!(
                "@{}\
                 D=A\
                 @SP\
                 A=M\
                 M=D\
                 D=A+1\
                 @SP\
                 M=D",
                index
            ),
            _ => unimplemented!("TODO"),
        };

        self.target.write_all(instructions.as_bytes());
    }

    // pub fn close() {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn write_arithmetic_test() {
        let mut output = Vec::<u8>::new();
        let mut writer = CodeWriter::new(&mut output);

        writer.write_arithmetic("");
        assert_eq!("add", str::from_utf8(&output).unwrap())
    }
}
