fn dest(mnemonic: Option<String>) -> String {
    match mnemonic.as_deref() {
        None => "000".to_string(),
        Some("M") => "001".to_string(),
        Some("D") => "010".to_string(),
        Some("MD") => "011".to_string(),
        Some("A") => "100".to_string(),
        Some("AM") => "101".to_string(),
        Some("AD") => "110".to_string(),
        Some("AMD") => "111".to_string(),
        Some(m) => panic!("unexpected mnemonic was given: {:?}", m),
    }
}

fn comp(mnemonic: String) -> String {
    match mnemonic.as_str() {
        "0" => "101010",
        "1" => "111111",
        "-1" => "111010",
        "D" => "001100",
        "A" | "M" => "110000",
        "!D" => "001101",
        "!A" | "!M" => "110001",
        "-D" => "001111",
        "-A" | "-M" => "110011",
        "D+1" => "011111",
        "A+1" | "M+1" => "110111",
        "D-1" => "001110",
        "A-1" | "M-1" => "110010",
        "D+A" | "D+M" => "000010",
        "D-A" | "D-M" => "010011",
        "A-D" | "M-D" => "000111",
        "D&A" | "D&M" => "000000",
        "D|A" | "D|M" => "010101",
        _ => panic!("unexpected mnemonic was given: {:?}", mnemonic),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dest_test() {
        assert_eq!("000", dest(None));
        assert_eq!("111", dest(Some("AMD".to_string())));
    }
}
