//! Tests for importing, aliasing, and namespacing.
//!
//! Each test compiles and type-checks .qu files under tests/import_aliasing/.
//! Entry files are named test_*.qu; helper modules (e.g. math.qu) live in the same dir.

use quorc::frontend::{lexer::Lexer, parser::Parser};
use quorc::midend::analyzer::TypeChecker;
use std::path::Path;

fn manifest_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn compile_and_typecheck(entry_name: &str) -> Result<Vec<quorc::frontend::ast::Stmt>, String> {
    let path = manifest_dir().join("tests").join("import_aliasing").join(entry_name);
    let source = std::fs::read_to_string(&path).map_err(|e| format!("read file: {e}"))?;
    let tokens = Lexer::new(source).tokenize().map_err(|e| format!("lex: {e:?}"))?;
    let program = Parser::new(tokens).parse().map_err(|e| format!("parse: {e:?}"))?;
    let mut tc = TypeChecker::default();
    let (program, module_count) = tc.process_program(program, &path);
    TypeChecker::analyze_program(program, &path, module_count, tc.aliases)
}

#[test]
fn basic_import_with_alias() {
    let result = compile_and_typecheck("test_basic_alias.qu");
    assert!(result.is_ok(), "basic import with alias should typecheck: {:?}", result.err());
}

#[test]
fn namespace_call_imported_function() {
    let result = compile_and_typecheck("test_namespace_call.qu");
    assert!(result.is_ok(), "namespace call (m::add) should typecheck: {:?}", result.err());
}

#[test]
fn builtin_from_entry_module() {
    let result = compile_and_typecheck("test_builtin_entry.qu");
    assert!(result.is_ok(), "malloc/exit from entry should typecheck: {:?}", result.err());
}

#[test]
fn builtin_from_imported_module() {
    let result = compile_and_typecheck("test_builtin_imported.qu");
    assert!(result.is_ok(), "malloc from imported module should typecheck: {:?}", result.err());
}

#[test]
fn struct_from_imported_module() {
    let result = compile_and_typecheck("test_struct_imported.qu");
    assert!(result.is_ok(), "struct type from imported module should typecheck: {:?}", result.err());
}

#[test]
fn multiple_imports_different_aliases() {
    let result = compile_and_typecheck("test_multi_import.qu");
    assert!(result.is_ok(), "multiple imports with different aliases: {:?}", result.err());
}

#[test]
fn same_name_different_modules() {
    let result = compile_and_typecheck("test_same_name.qu");
    assert!(result.is_ok(), "same name in different modules (foo::x, bar::x): {:?}", result.err());
}

#[test]
fn nested_import() {
    let result = compile_and_typecheck("test_nested_import.qu");
    assert!(result.is_ok(), "A imports B, B imports C: {:?}", result.err());
}

#[test]
fn import_no_alias_uses_stem() {
    let result = compile_and_typecheck("test_import_no_alias.qu");
    assert!(result.is_ok(), "import without | alias uses file stem: {:?}", result.err());
}

#[test]
fn local_shadows_module_name() {
    let result = compile_and_typecheck("test_local_shadow.qu");
    assert!(result.is_ok(), "local variable shadows module-level name: {:?}", result.err());
}
