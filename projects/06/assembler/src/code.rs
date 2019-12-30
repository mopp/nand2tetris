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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dest_test() {
        assert_eq!("000", dest(None));
        assert_eq!("111", dest(Some("AMD".to_string())));
    }
}
