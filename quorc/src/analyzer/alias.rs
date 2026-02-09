use std::collections::HashMap;

use crate::frontend::ast::*;

#[derive(Clone, Debug, Default)]
pub struct ModuleSymbols {
    pub structs: HashMap<String, bool>,
    pub struct_fields: HashMap<String, Vec<(String, Type)>>,
    pub functions: HashMap<String, (Vec<Type>, Type, Vec<String>)>,
    pub globals: HashMap<String, Type>,
}

#[derive(Clone, Debug, Default)]
pub struct QuorFile {
    pub file: CanonicalFile,
    pub symbols: ModuleSymbols,
    pub aliases: HashMap<Alias, CanonicalFile>,
}

pub type CanonicalFile = String;
pub type Alias = String;

#[derive(Default, Debug)]
pub struct AliasManager {
    pub module_registry: HashMap<CanonicalFile, QuorFile>,
    pub module_ids: HashMap<CanonicalFile, usize>,
    module_count: usize,
}

impl AliasManager {
    pub fn register_file(
        &mut self,
        canonical_name: String,
        symbols: ModuleSymbols,
        aliases: HashMap<Alias, CanonicalFile>,
    ) {
        let id = self.module_count;
        let file = QuorFile {
            file: canonical_name.clone(),
            symbols,
            aliases,
        };
        self.module_ids.insert(canonical_name.clone(), id);
        self.module_registry.insert(canonical_name, file);
        self.module_count += 1;
    }

    pub fn is_viable_struct(
        &self,
        file_that_is_being_parsed: &CanonicalFile,
        namespace: &Alias,
        struct_name: &String,
    ) -> bool {
        let file_name = self
            .module_registry
            .get(file_that_is_being_parsed)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .aliases
            .get(namespace)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            });
        self.module_registry
            .get(file_name)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .symbols
            .structs
            .contains_key(struct_name)
    }

    pub fn is_viable_func(
        &self,
        file_that_is_being_parsed: &CanonicalFile,
        namespace: &Alias,
        func_name: &String,
    ) -> bool {
        let file_name = self
            .module_registry
            .get(file_that_is_being_parsed)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .aliases
            .get(namespace)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            });
        self.module_registry
            .get(file_name)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .symbols
            .functions
            .contains_key(func_name)
    }

    pub fn is_viable_global(
        &self,
        file_that_is_being_parsed: &CanonicalFile,
        namespace: &Alias,
        global_name: &String,
    ) -> bool {
        let file_name = self
            .module_registry
            .get(file_that_is_being_parsed)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .aliases
            .get(namespace)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            });
        self.module_registry
            .get(file_name)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .symbols
            .functions
            .contains_key(global_name)
    }

    pub fn get_struct_fields(
        &self,
        file_that_is_being_parsed: &CanonicalFile,
        namespace: &Alias,
        name: &String,
    ) -> Option<&Vec<(String, Type)>> {
        let file_name = self
            .module_registry
            .get(file_that_is_being_parsed)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .aliases
            .get(namespace)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            });
        self.module_registry
            .get(file_name)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .symbols
            .struct_fields
            .get(name)
    }

    pub fn get_struct_is_union(
        &self,
        file_that_is_being_parsed: &CanonicalFile,
        namespace: &Alias,
        name: &String,
    ) -> Option<&bool> {
        let file_name = self
            .module_registry
            .get(file_that_is_being_parsed)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .aliases
            .get(namespace)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            });
        self.module_registry
            .get(file_name)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .symbols
            .structs
            .get(name)
    }

    pub fn get_function_info(
        &self,
        file_that_is_being_parsed: &CanonicalFile,
        namespace: &Alias,
        name: &String,
    ) -> Option<&(Vec<Type>, Type, Vec<String>)> {
        let file_name = self
            .module_registry
            .get(file_that_is_being_parsed)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .aliases
            .get(namespace)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            });
        self.module_registry
            .get(file_name)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .symbols
            .functions
            .get(name)
    }

    pub fn get_global(
        &self,
        file_that_is_being_parsed: &CanonicalFile,
        namespace: &Alias,
        name: &String,
    ) -> Option<&Type> {
        let file_name = self
            .module_registry
            .get(file_that_is_being_parsed)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .aliases
            .get(namespace)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            });
        self.module_registry
            .get(file_name)
            .unwrap_or_else(|| {
                panic!("file: {file_that_is_being_parsed} does not contain import: {namespace}")
            })
            .symbols
            .globals
            .get(name)
    }
}
