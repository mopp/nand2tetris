use super::parser::{Command, Index, Indirect, MappedMemory, Segment};
use std::io::prelude::*;
use std::io::Result;

const INITIAL_GLOBAL_STACK_ADDR: u16 = 256;
const INITIAL_FUNCTION_NAME: &str = "Sys.init";

pub struct CodeWriter<'a, W: Write> {
    target: &'a mut W,
    label_counter: usize,
}

impl<'a, W: Write> CodeWriter<'a, W> {
    pub fn new(target: &'a mut W) -> Self {
        Self {
            target,
            label_counter: 0,
        }
    }

    pub fn write_bootstrap_code(&mut self) -> Result<()> {
        let code = format!(
            "// Bootstrap code\n\
            @{initial_global_stack_addr}\n\
            D=A\n\
            @SP\n\
            M=D // SP = 256\n",
            initial_global_stack_addr = INITIAL_GLOBAL_STACK_ADDR
        );
        self.target.write_all(code.as_bytes())?;

        self.put(
            "Bootstrap.vm",
            &Command::Call(INITIAL_FUNCTION_NAME.to_string(), 0),
        )
    }

    pub fn put(&mut self, file_name: &str, command: &Command) -> Result<()> {
        use Command::*;
        let mut instructions = match command {
            Add => self.generate_add(),
            Sub => self.generate_sub(),
            Neg => self.generate_neg(),
            And => self.generate_and(),
            Or => self.generate_or(),
            Not => self.generate_not(),
            Eq => self.generate_eq(),
            Lt => self.generate_lt(),
            Gt => self.generate_gt(),
            Push(segment, index) => self.generate_push(segment, *index, file_name),
            Pop(segment, index) => self.generate_pop(segment, *index, file_name),
            Label(name) => self.generate_label(name),
            Goto(name) => self.generate_goto(name),
            IfGoto(name) => self.generate_if_goto(name),
            Function(name, argc) => self.generate_function(name, *argc),
            Return => self.generate_return(),
            Call(name, argc) => self.generate_call(name, *argc),
        };

        instructions.push('\n');

        self.target.write_all(instructions.as_bytes())
    }

    fn generate_add(&self) -> String {
        "\
        // add\n\
        @SP\n\
        M=M-1 // *SP -= 1\n\
        A=M\n\
        D=M // D = **SP\n\
        @SP\n\
        A=M-1\n\
        M=M+D // *(*SP - 1) += D\n\
        "
        .to_string()
    }

    fn generate_sub(&self) -> String {
        "\
        // sub\n\
        @SP\n\
        M=M-1 // *SP -= 1\n\
        A=M\n\
        D=M // D = **SP\n\
        @SP\n\
        A=M-1\n\
        M=M-D // *(*SP - 1) -= D\n\
        "
        .to_string()
    }

    fn generate_neg(&self) -> String {
        "\
        // neg\n\
        @SP\n\
        A=M - 1\n\
        M=!M // **SP = !**SP\n\
        M=M+1 // **SP += 1\n\
        "
        .to_string()
    }

    fn generate_and(&self) -> String {
        "\
        // and\n\
        @SP\n\
        M=M-1 // *SP -= 1\n\
        A=M\n\
        D=M // D = **SP\n\
        @SP\n\
        A=M-1\n\
        M=M&D // *(*SP - 1) &= D\n\
        "
        .to_string()
    }

    fn generate_or(&self) -> String {
        "\
        // or\n\
        @SP\n\
        M=M-1 // *SP -= 1\n\
        A=M\n\
        D=M // D = **SP\n\
        @SP\n\
        A=M-1\n\
        M=M|D // *(*SP - 1) |= D\n\
        "
        .to_string()
    }

    fn generate_not(&self) -> String {
        "\
        // not\n\
        @SP\n\
        A=M - 1\n\
        M=!M // **SP = !**SP\n\
        "
        .to_string()
    }

    fn generate_eq(&mut self) -> String {
        format!(
            "\
            // eq\n\
            @SP\n\
            M=M-1 // *SP -= 1\n\
            A=M\n\
            D=M // D = **SP\n\
            @SP\n\
            A=M-1\n\
            D=M-D // D = *(*SP - 1) - D\n\
            @_LABEL{n}_TRUE\n\
            D;JEQ\n\
            D=0\n\
            @_LABEL{n}_END\n\
            D;JMP\n\
            (_LABEL{n}_TRUE)\n\
            D=-1\n\
            (_LABEL{n}_END)\n\
            @SP\n\
            A=M-1\n\
            M=D // *(*SP - 1) = D\n\
            ",
            n = self.use_label_counter()
        )
    }

    fn generate_lt(&mut self) -> String {
        format!(
            "\
            // lt\n\
            @SP\n\
            M=M-1 // *SP -= 1\n\
            A=M\n\
            D=M // D = **SP\n\
            @SP\n\
            A=M-1\n\
            D=M-D // D = *(*SP - 1) - D\n\
            @_LABEL{n}_TRUE\n\
            D;JLT\n\
            D=0\n\
            @_LABEL{n}_END\n\
            D;JMP\n\
            (_LABEL{n}_TRUE)\n\
            D=-1\n\
            (_LABEL{n}_END)\n\
            @SP\n\
            A=M-1\n\
            M=D // *(*SP - 1) = D\n\
            ",
            n = self.use_label_counter()
        )
    }

    fn generate_gt(&mut self) -> String {
        format!(
            "\
            // gt\n\
            @SP\n\
            M=M-1 // *SP -= 1\n\
            A=M\n\
            D=M // D = **SP\n\
            @SP\n\
            A=M-1\n\
            D=M-D // D = *(*SP - 1) - D\n\
            @_LABEL{n}_TRUE\n\
            D;JGT\n\
            D=0\n\
            @_LABEL{n}_END\n\
            D;JMP\n\
            (_LABEL{n}_TRUE)\n\
            D=-1\n\
            (_LABEL{n}_END)\n\
            @SP\n\
            A=M-1\n\
            M=D // *(*SP - 1) = D\n\
            ",
            n = self.use_label_counter()
        )
    }

    fn generate_push(&mut self, segment: &Segment, index: Index, file_name: &str) -> String {
        use MappedMemory::*;
        match segment {
            Segment::Indirect(ref segment) => format!(
                "\
                // push {segment} {index}\n\
                @{register_name}\n\
                D=M\n\
                @{index}\n\
                A=D+A\n\
                D=M // D = &{segment}[{index}]\n\
                @SP\n\
                M=M+1 // *SP += 1\n\
                A=M-1\n\
                M=D // *(*SP - 1) = D\n\
                ",
                segment = segment,
                index = index,
                register_name = self.get_indirect_register_name(segment),
            ),
            Segment::MappedMemory(Temp) => format!(
                "\
                // push temp {index}\n\
                @5\n\
                D=A\n\
                @{index}\n\
                A=A+D\n\
                D=M // D = temp[{index}]\n\
                @SP\n\
                M=M+1 // *SP += 1\n\
                A=M-1\n\
                M=D // *(*SP - 1) = D\n\
                ",
                index = index
            ),
            Segment::MappedMemory(Pointer) => format!(
                "\
                // push pointer {index}\n\
                @{register_name}
                D=M // D = pointer[{index}]\n\
                @SP
                M=M+1 // *SP += 1\n\
                A=M-1\n\
                M=D // **SP = D\n
                ",
                index = index,
                register_name = self.get_pointer_resigter_name(index),
            ),
            Segment::Static => format!(
                "\
                // push static {index}\n\
                @{file_name}.{index}
                D=M
                @SP
                M=M+1 // *SP += 1\n\
                A=M-1\n\
                M=D // **SP = D\n\
                ",
                index = index,
                file_name = file_name
            ),
            Segment::Constant => format!(
                "\
                // push constant {index}\n\
                @{index}\n\
                D=A // D = {index}\n\
                @SP\n\
                A=M\n\
                M=D // **SP = D\n\
                @SP\n\
                M=M+1 // *SP += 1\n\
                ",
                index = index
            ),
        }
    }

    fn generate_pop(&mut self, segment: &Segment, index: Index, file_name: &str) -> String {
        use MappedMemory::*;
        match segment {
            Segment::Indirect(segment) => format!(
                "\
                // pop {segment} {index}\n\
                @{register_name}\n\
                D=M\n\
                @{index}\n\
                D=D+A\n\
                @R13\n\
                M=D // R13 = &{segment}[{index}]\n\
                @SP\n\
                M=M-1 // *SP -= 1\n\
                A=M\n\
                D=M // D = **SP\n\
                @R13\n\
                A=M\n\
                M=D // *R13 = D\n\
                ",
                segment = segment,
                index = index,
                register_name = self.get_indirect_register_name(segment)
            ),
            Segment::MappedMemory(Temp) => format!(
                "\
                // pop temp {index}\n\
                @5\n\
                D=A\n\
                @{index}\n\
                D=D+A\n\
                @R13\n\
                M=D // R13 = &temp[{index}]\n\
                @SP\n\
                M=M-1 // *SP -= 1\n\
                A=M\n\
                D=M // D = **SP\n\
                @R13\n\
                A=M\n\
                M=D // *R13 = D\n\
                ",
                index = index
            ),
            Segment::MappedMemory(Pointer) => format!(
                "\
                // pop pointer {index}\n\
                @SP\n\
                M=M-1 // *SP -= 1\n\
                A=M\n\
                D=M // D = **SP\n\
                @{register_name}\n\
                M=D // pointer[{index}] = D\n\
                ",
                index = index,
                register_name = self.get_pointer_resigter_name(index)
            ),
            Segment::Static => format!(
                "\
                // pop static {index}\n\
                @SP\n\
                M=M-1 // *SP -= 1\n\
                A=M\n\
                D=M // D = **SP\n\
                @{file_name}.{index}\n\
                M=D // pointer[{index}] = D\n\
                ",
                index = index,
                file_name = file_name
            ),
            Segment::Constant => panic!("pop constant N is invalid."),
        }
    }

    fn generate_label(&self, name: &String) -> String {
        format!("({})\n", name)
    }

    fn generate_goto(&self, name: &String) -> String {
        format!(
            "\
            // goto {name}\n\
            @{name}\n\
            0;JMP\n\
            ",
            name = name,
        )
    }

    fn generate_if_goto(&self, name: &String) -> String {
        format!(
            "\
            // if-goto {name}\n\
            @SP\n\
            M=M-1 // *SP -= 1\n\
            A=M\n\
            D=M\n\
            @{name}\n\
            D;JNE\n\
            ",
            name = name,
        )
    }

    fn generate_function(&self, name: &String, argc: u16) -> String {
        let mut body = String::new();
        if argc != 0 {
            body.push_str(
                format!(
                    "\
                    @{argc}\n\
                    D=A\n\
                    @SP\n\
                    M=M+D\n\
                    @LCL\n\
                    A=M\n\
                    ",
                    argc = argc
                )
                .as_str(),
            );
            for _ in 0..argc {
                body.push_str(
                    "\
                M=0\n\
                A=A+1\n\
                ",
                );
            }
        }

        format!(
            "\
            // function {name} {argc}\n\
            ({name})\n\
            {body}\
            ",
            name = name,
            argc = argc,
            body = body
        )
    }

    fn generate_return(&self) -> String {
        "\
        // return\n\
        @5\n\
        D=A\n\
        @LCL\n\
        A=M-D\n\
        D=M\n\
        @R13\n\
        M=D // *R13 = return_addr\n\
        @SP\n\
        A=M-1\n\
        D=M\n\
        @ARG\n\
        A=M\n\
        M=D // **ARG = return_value\n\
        @ARG\n\
        D=M+1\n\
        @SP\n\
        M=D // *SP = *ARG + 1\n\
        @LCL\n\
        D=M // D = *LCL\n\
        D=D-1\n\
        @R14\n\
        M=D\n\
        A=D\n\
        D=M\n\
        @THAT\n\
        M=D // *THAT = (**LCL - 1)\n\
        @R14\n\
        M=M-1\n\
        A=M\n\
        D=M\n\
        @THIS\n\
        M=D // *THIS = (**LCL - 2)\n\
        @R14\n\
        M=M-1\n\
        A=M\n\
        D=M\n\
        @ARG\n\
        M=D // *ARG = (**LCL - 3)\n\
        @R14\n\
        M=M-1\n\
        A=M\n\
        D=M\n\
        @LCL\n\
        M=D // *LCL = (**LCL - 4)\n\
        @R13\n\
        A=M\n\
        0;JMP // Jump to return address.\n\
        "
        .to_string()
    }

    fn generate_call(&mut self, name: &String, argc: u16) -> String {
        format!(
            "\
            // call {name} {argc}\n\
            @return_addr{n}\n\
            D=A\n\
            @SP\n\
            M=M+1\n\
            A=M-1\n\
            M=D // push return address\n\
            @LCL\n\
            D=M\n\
            @SP\n\
            M=M+1\n\
            A=M-1\n\
            M=D // push LCL\n\
            @ARG\n\
            D=M\n\
            @SP\n\
            M=M+1\n\
            A=M-1\n\
            M=D // push ARG\n\
            @THIS\n\
            D=M\n\
            @SP\n\
            M=M+1\n\
            A=M-1\n\
            M=D // push THIS\n\
            @THAT\n\
            D=M\n\
            @SP\n\
            M=M+1\n\
            A=M-1\n\
            M=D // push THAT\n\
            @SP\n\
            D=M\n\
            @{argc}\n\
            D=D-A\n\
            @5\n\
            D=D-A\n\
            @ARG\n\
            M=D // *ARG = SP - n - 5\n\
            @SP\n\
            D=M\n\
            @LCL\n\
            M=D // *LCL = SP\n\
            @{name}\n\
            0;JMP // goto {name}\n\
            (return_addr{n})\n\
            ",
            name = name,
            argc = argc,
            n = self.use_label_counter()
        )
    }

    fn get_indirect_register_name(&self, segment: &Indirect) -> &'static str {
        match segment {
            Indirect::Local => "LCL",
            Indirect::Argument => "ARG",
            Indirect::This => "THIS",
            Indirect::That => "THAT",
        }
    }

    fn get_pointer_resigter_name(&self, index: Index) -> &'static str {
        match index {
            0 => "THIS",
            1 => "THAT",
            _ => panic!("unexpected pointer segment: {}", index),
        }
    }

    fn use_label_counter(&mut self) -> usize {
        self.label_counter += 1;
        self.label_counter
    }
}
