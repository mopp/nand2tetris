use std::collections::HashMap;

#[derive(Debug)]
pub struct SymbolTable {
    class_scope: HashMap<Identifier, SymbolInfo>,
    subroutine_scope: HashMap<Identifier, SymbolInfo>,
    count: HashMap<Kind, usize>,
}

type Identifier = String;
type Type = String;

#[derive(Debug)]
struct SymbolInfo {
    itype: Type,
    kind: Kind,
    index: usize,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
    Static,
    Field,
    Arg,
    Var,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            class_scope: HashMap::new(),
            subroutine_scope: HashMap::new(),
            count: HashMap::with_capacity(4),
        }
    }

    pub fn start_subroutine(&mut self) {
        self.subroutine_scope.clear();
    }

    pub fn define(&mut self, identifier: Identifier, itype: Type, kind: Kind) {
        let index = self.var_count(kind);
        let info = SymbolInfo { itype, kind, index };

        println!("define {:?} -> {:?}", identifier, info);

        match kind {
            Kind::Static | Kind::Field => {
                self.class_scope.insert(identifier, info);
            }
            _ => {
                self.subroutine_scope.insert(identifier, info);
            }
        }

        self.count.insert(kind, index + 1);
    }

    pub fn kind_of(&self, identifier: Identifier) -> Option<Kind> {
        self.info_of(identifier).map(|info| info.kind)
    }

    pub fn type_of(&self, identifier: Identifier) -> Option<Type> {
        self.info_of(identifier).map(|info| info.itype.clone())
    }

    pub fn index_of(&self, identifier: Identifier) -> Option<usize> {
        self.info_of(identifier).map(|info| info.index)
    }

    fn info_of(&self, identifier: Identifier) -> Option<&SymbolInfo> {
        if let Some(info) = self.subroutine_scope.get(&identifier) {
            return Some(info);
        }

        if let Some(info) = self.class_scope.get(&identifier) {
            return Some(info);
        }

        None
    }

    fn var_count(&self, kind: Kind) -> usize {
        match self.count.get(&kind) {
            None => 0,
            Some(n) => *n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();

        table.define("hoge".to_string(), "int".to_string(), Kind::Static);

        table.start_subroutine();

        println!("{:?}", table);

        assert_eq!(Some(Kind::Static), table.kind_of("hoge".to_string()));
        assert_eq!(Some("int".to_string()), table.type_of("hoge".to_string()));
        assert_eq!(Some(0), table.index_of("hoge".to_string()));

        assert_eq!(None, table.index_of("piyo".to_string()));
    }
}
