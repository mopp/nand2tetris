use std::collections::HashMap;

type Symbol = String;
type Address = u16;

#[derive(Debug)]
pub struct SymbolTable {
    table: HashMap<Symbol, Address>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table: HashMap<Symbol, Address> = HashMap::with_capacity(32);

        for (k, v) in [
            ("SP", 0),
            ("LCL", 1),
            ("ARG", 2),
            ("THIS", 3),
            ("THAT", 4),
            ("R0", 0),
            ("R1", 1),
            ("R2", 2),
            ("R3", 3),
            ("R4", 4),
            ("R5", 5),
            ("R6", 6),
            ("R7", 7),
            ("R8", 8),
            ("R9", 9),
            ("R10", 10),
            ("R11", 11),
            ("R12", 12),
            ("R13", 13),
            ("R14", 14),
            ("R15", 15),
            ("SCREEN", 0x4000),
            ("KBD", 0x6000),
        ]
        .iter()
        {
            table.insert(k.to_string(), *v);
        }

        Self { table }
    }

    pub fn add_entry(&mut self, symbol: Symbol, address: Address) {
        self.table.insert(symbol, address);
    }

    // pub fn contains(&self, symbol: &Symbol) -> bool {
    //     self.table.contains_key(symbol)
    // }

    pub fn get_address(&self, symbol: &Symbol) -> Option<Address> {
        self.table.get(symbol).cloned()
    }
}
