fn dest(mnemonic: Option<String>) -> String {
    match mnemonic.as_deref() {
        None => "000",
        Some("M") => "001",
        Some("D") => "010",
        Some("MD") => "011",
        Some("A") => "100",
        Some("AM") => "101",
        Some("AD") => "110",
        Some("AMD") => "111",
        Some(m) => panic!("unexpected mnemonic was given: {:?}", m),
    }
    .to_string()
}

fn comp(mnemonic: String) -> String {
    match mnemonic.as_str() {
        "0" => "0101010",
        "1" => "0111111",
        "-1" => "0111010",
        "D" => "0001100",
        "A" => "0110000",
        "M" => "1110000",
        "!D" => "0001101",
        "!A" => "0110001",
        "!M" => "1110001",
        "-D" => "0001111",
        "-A" => "0110011",
        "-M" => "1110011",
        "D+1" => "0011111",
        "A+1" => "0110111",
        "M+1" => "1110111",
        "D-1" => "0001110",
        "A-1" => "0110010",
        "D+A" => "0000010",
        "D-A" => "0010011",
        "A-D" => "0000111",
        "D&A" => "0000000",
        "D|A" => "0010101",
        "M-1" => "1110010",
        "D+M" => "1000010",
        "D-M" => "1010011",
        "M-D" => "1000111",
        "D&M" => "1000000",
        "D|M" => "1010101",
        _ => panic!("unexpected mnemonic was given: {:?}", mnemonic),
    }
    .to_string()
}

fn jump(mnemonic: Option<String>) -> String {
    match mnemonic.as_deref() {
        None => "000",
        Some("JGT") => "001",
        Some("JEQ") => "010",
        Some("JGE") => "011",
        Some("JLT") => "100",
        Some("JNE") => "101",
        Some("JLE") => "110",
        Some("JMP") => "111",
        Some(m) => panic!("unexpected mnemonic was given: {:?}", m),
    }
    .to_string()
}
