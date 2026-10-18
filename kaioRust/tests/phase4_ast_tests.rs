use kaioken::scanner::{
    AstParser, Language, ParseStatus, SymbolKind,
};

#[test]
fn test_rust_functions_and_visibility() {
    let code = r#"
pub async fn fetch_user(id: u64) -> Result<String, Error> {
    Ok("Alice".to_string())
}

pub(crate) fn internal_helper() -> bool {
    true
}

fn private_worker() {
    // nothing
}
"#;

    let res = AstParser::parse_source(code, Language::Rust).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);
    assert!(!res.has_syntax_errors);
    assert_eq!(res.symbols.len(), 3);

    let fetch_user = res.symbols.iter().find(|s| s.name == "fetch_user").unwrap();
    assert_eq!(fetch_user.kind, SymbolKind::Function);
    assert_eq!(fetch_user.visibility, "pub");
    assert!(fetch_user.signature.as_ref().unwrap().contains("pub async fn fetch_user"));
    assert_eq!(fetch_user.line_start, 2);

    let internal = res.symbols.iter().find(|s| s.name == "internal_helper").unwrap();
    assert_eq!(internal.visibility, "pub(crate)");

    let private = res.symbols.iter().find(|s| s.name == "private_worker").unwrap();
    assert_eq!(private.visibility, "private");

    assert!(res.exports.contains(&"fetch_user".to_string()));
    assert!(res.exports.contains(&"internal_helper".to_string()));
    assert!(!res.exports.contains(&"private_worker".to_string()));
}

#[test]
fn test_rust_structs_enums_and_traits() {
    let code = r#"
pub struct UserRecord {
    pub id: u64,
    pub username: String,
}

pub enum ConnectionState {
    Connected,
    Disconnected,
}

pub trait Repository {
    fn find_by_id(&self, id: u64) -> Option<UserRecord>;
}
"#;

    let res = AstParser::parse_source(code, Language::Rust).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);
    assert_eq!(res.symbols.len(), 4); // UserRecord, ConnectionState, Repository, find_by_id

    let struct_sym = res.symbols.iter().find(|s| s.name == "UserRecord").unwrap();
    assert_eq!(struct_sym.kind, SymbolKind::Struct);
    assert_eq!(struct_sym.visibility, "pub");

    let enum_sym = res.symbols.iter().find(|s| s.name == "ConnectionState").unwrap();
    assert_eq!(enum_sym.kind, SymbolKind::Enum);
    assert_eq!(enum_sym.visibility, "pub");

    let trait_sym = res.symbols.iter().find(|s| s.name == "Repository").unwrap();
    assert_eq!(trait_sym.kind, SymbolKind::Trait);
    assert_eq!(trait_sym.visibility, "pub");

    let method_sym = res.symbols.iter().find(|s| s.name == "find_by_id").unwrap();
    assert_eq!(method_sym.kind, SymbolKind::Method);
    assert_eq!(method_sym.parent, Some("Repository".to_string()));
}

#[test]
fn test_rust_impl_and_methods() {
    let code = r#"
pub struct Calculator;

impl Calculator {
    pub fn new() -> Self {
        Calculator
    }

    fn calculate(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

impl std::fmt::Display for Calculator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Calculator")
    }
}
"#;

    let res = AstParser::parse_source(code, Language::Rust).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    let calc_struct = res.symbols.iter().find(|s| s.name == "Calculator" && s.kind == SymbolKind::Struct).unwrap();
    assert_eq!(calc_struct.kind, SymbolKind::Struct);

    let impl_block = res.symbols.iter().find(|s| s.name == "impl Calculator").unwrap();
    assert_eq!(impl_block.kind, SymbolKind::Impl);

    let new_method = res.symbols.iter().find(|s| s.name == "new").unwrap();
    assert_eq!(new_method.kind, SymbolKind::Method);
    assert_eq!(new_method.visibility, "pub");
    assert_eq!(new_method.parent, Some("impl Calculator".to_string()));

    let calc_method = res.symbols.iter().find(|s| s.name == "calculate").unwrap();
    assert_eq!(calc_method.kind, SymbolKind::Method);
    assert_eq!(calc_method.visibility, "private");
    assert_eq!(calc_method.parent, Some("impl Calculator".to_string()));

    let trait_impl = res.symbols.iter().find(|s| s.name == "impl std::fmt::Display for Calculator").unwrap();
    assert_eq!(trait_impl.kind, SymbolKind::Impl);

    let fmt_method = res.symbols.iter().find(|s| s.name == "fmt").unwrap();
    assert_eq!(fmt_method.kind, SymbolKind::Method);
    assert_eq!(fmt_method.parent, Some("impl std::fmt::Display for Calculator".to_string()));
}

#[test]
fn test_rust_imports_and_exports() {
    let code = r#"
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
pub use crate::error::Result;

pub fn public_api() {}
"#;

    let res = AstParser::parse_source(code, Language::Rust).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);
    assert_eq!(res.imports_raw.len(), 3);
    assert!(res.imports_raw.iter().any(|i| i.contains("std::path::{Path, PathBuf}")));
    assert!(res.imports_raw.iter().any(|i| i.contains("serde::{Deserialize, Serialize}")));
    assert!(res.imports_raw.iter().any(|i| i.contains("crate::error::Result")));

    assert!(res.exports.contains(&"public_api".to_string()));
}

#[test]
fn test_syntax_error_recovery_partial_status() {
    // Source file with a valid function, then broken code, then another valid struct and function
    let code = r#"
pub fn first_valid_function() -> i32 {
    42
}

// Broken syntax error: incomplete declaration
pub fn broken_syntax( {
    invalid = := 123;
}

pub struct SurvivingStruct {
    pub value: String,
}

pub fn second_valid_function() -> bool {
    true
}
"#;

    let res = AstParser::parse_source(code, Language::Rust).unwrap();
    // Syntax error must be detected, setting ParseStatus::Partial
    assert!(res.has_syntax_errors, "Must detect presence of syntax error");
    assert_eq!(res.parse_status, ParseStatus::Partial, "ParseStatus must be Partial");

    // Robust error recovery: tree-sitter must STILL extract valid symbols around the error!
    let names: Vec<&str> = res.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"first_valid_function"), "Must recover first_valid_function");
    assert!(names.contains(&"SurvivingStruct"), "Must recover SurvivingStruct");
    assert!(names.contains(&"second_valid_function"), "Must recover second_valid_function");
}

#[test]
fn test_unsupported_language_graceful_handling() {
    let markdown = "# Hello World\nThis is docs.";
    let res = AstParser::parse_source(markdown, Language::Markdown).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);
    assert!(res.symbols.is_empty());
    assert!(!res.has_syntax_errors);
}
