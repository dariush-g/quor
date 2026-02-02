pub mod aarch64;
pub mod regalloc;
pub mod x86_64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymId(pub usize);

#[derive(Default)]
pub struct Symbols {
    map: std::collections::HashMap<String, SymId>,
    vec: Vec<String>,
}

impl Symbols {
    pub fn intern(&mut self, s: &str) -> SymId {
        if let Some(id) = self.map.get(s) {
            return *id;
        }
        let id = SymId(self.vec.len());
        self.vec.push(s.to_owned());
        self.map.insert(s.to_owned(), id);
        id
    }

    pub fn mangle_fn(sym: SymId, symbols: &Symbols) -> String {
        let name = symbols.get(sym);
        if cfg!(target_os = "macos") {
            format!("_{}", name)
        } else {
            name.to_string()
        }
    }

    pub fn get(&self, id: SymId) -> &str {
        &self.vec[id.0]
    }
}
