use kaioken::scanner::{AstParser, Language, ParseStatus, SymbolKind};
use std::path::Path;

/// 1. test_typescript_functions:
/// Extract `function` and `arrow` declarations from TypeScript
#[test]
fn test_typescript_functions() {
    let code = r#"
export function calculateSum(a: number, b: number): number {
    return a + b;
}

function helperInternal(): void {}

export const multiply = (x: number, y: number): number => x * y;

const localArrow = () => { return "hello"; };
"#;

    let res = AstParser::parse_source(code, Language::TypeScript).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);
    assert!(!res.has_syntax_errors);

    let func_names: Vec<&str> = res.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(func_names.contains(&"calculateSum"));
    assert!(func_names.contains(&"helperInternal"));
    assert!(func_names.contains(&"multiply"));
    assert!(func_names.contains(&"localArrow"));

    let calc = res
        .symbols
        .iter()
        .find(|s| s.name == "calculateSum")
        .unwrap();
    assert_eq!(calc.kind, SymbolKind::Function);
    assert_eq!(calc.visibility, "export");

    let helper = res
        .symbols
        .iter()
        .find(|s| s.name == "helperInternal")
        .unwrap();
    assert_eq!(helper.kind, SymbolKind::Function);
    assert_eq!(helper.visibility, "private");

    let mul = res.symbols.iter().find(|s| s.name == "multiply").unwrap();
    assert_eq!(mul.kind, SymbolKind::Arrow);
    assert_eq!(mul.visibility, "export");

    let local = res.symbols.iter().find(|s| s.name == "localArrow").unwrap();
    assert_eq!(local.kind, SymbolKind::Arrow);
    assert_eq!(local.visibility, "private");
}

/// 2. test_typescript_classes:
/// Extract class declarations, class methods, and interfaces
#[test]
fn test_typescript_classes() {
    let code = r#"
export interface Repository<T> {
    findById(id: string): Promise<T>;
    save(item: T): Promise<void>;
}

export class SqlUserRepository implements Repository<User> {
    private connectionString: string;

    constructor(conn: string) {
        this.connectionString = conn;
    }

    public async findById(id: string): Promise<User> {
        return { id };
    }

    protected sanitize(raw: string): string {
        return raw.trim();
    }

    private logQuery(query: string): void {
        console.log(query);
    }
}
"#;

    let res = AstParser::parse_source(code, Language::TypeScript).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    // Interface
    let iface = res.symbols.iter().find(|s| s.name == "Repository").unwrap();
    assert_eq!(iface.kind, SymbolKind::Interface);
    assert_eq!(iface.visibility, "export");

    // Class
    let class_sym = res
        .symbols
        .iter()
        .find(|s| s.name == "SqlUserRepository")
        .unwrap();
    assert_eq!(class_sym.kind, SymbolKind::Class);
    assert_eq!(class_sym.visibility, "export");

    // Methods
    let find_method = res.symbols.iter().find(|s| s.name == "findById").unwrap();
    assert_eq!(find_method.kind, SymbolKind::Method);
    assert_eq!(find_method.parent.as_deref(), Some("SqlUserRepository"));
    assert_eq!(find_method.visibility, "public");

    let sanitize_method = res.symbols.iter().find(|s| s.name == "sanitize").unwrap();
    assert_eq!(sanitize_method.kind, SymbolKind::Method);
    assert_eq!(sanitize_method.visibility, "protected");

    let log_method = res.symbols.iter().find(|s| s.name == "logQuery").unwrap();
    assert_eq!(log_method.kind, SymbolKind::Method);
    assert_eq!(log_method.visibility, "private");
}

/// 3. test_typescript_enums:
/// Extract enum declarations and type aliases
#[test]
fn test_typescript_enums() {
    let code = r#"
export enum NetworkStatus {
    Connected = "CONNECTED",
    Disconnected = "DISCONNECTED"
}

enum InternalFlags {
    None = 0,
    Debug = 1
}

export type NetworkResponse = { status: NetworkStatus; code: number };
"#;

    let res = AstParser::parse_source(code, Language::TypeScript).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    let pub_enum = res
        .symbols
        .iter()
        .find(|s| s.name == "NetworkStatus")
        .unwrap();
    assert_eq!(pub_enum.kind, SymbolKind::Enum);
    assert_eq!(pub_enum.visibility, "export");

    let priv_enum = res
        .symbols
        .iter()
        .find(|s| s.name == "InternalFlags")
        .unwrap();
    assert_eq!(priv_enum.kind, SymbolKind::Enum);
    assert_eq!(priv_enum.visibility, "private");

    let type_alias = res
        .symbols
        .iter()
        .find(|s| s.name == "NetworkResponse")
        .unwrap();
    assert_eq!(type_alias.kind, SymbolKind::TypeAlias);
    assert_eq!(type_alias.visibility, "export");
}

/// 4. test_typescript_imports:
/// Extract ES6 import statements and raw paths
#[test]
fn test_typescript_imports() {
    let code = r#"
import { useState, useEffect } from 'react';
import axios from 'axios';
import * as path from 'path';
import { User, Token as AuthToken } from './types/user';
"#;

    let res = AstParser::parse_source(code, Language::TypeScript).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    assert!(res.imports_raw.contains(&"react".to_string()));
    assert!(res.imports_raw.contains(&"axios".to_string()));
    assert!(res.imports_raw.contains(&"path".to_string()));
    assert!(res.imports_raw.contains(&"./types/user".to_string()));

    assert!(res.imports.contains(&"useState".to_string()));
    assert!(res.imports.contains(&"useEffect".to_string()));
    assert!(res.imports.contains(&"axios".to_string()));
    assert!(res.imports.contains(&"path".to_string()));
    assert!(res.imports.contains(&"User".to_string()));
}

/// 5. test_tsx_components:
/// Extract React component declarations and TSX tags
#[test]
fn test_tsx_components() {
    let code = r#"
import React from 'react';

export interface HeaderProps {
    title: string;
}

export const HeaderBanner: React.FC<HeaderProps> = ({ title }) => {
    return (
        <div className="banner">
            <h1>{title}</h1>
        </div>
    );
};

export function Sidebar() {
    return <aside><nav>Menu</nav></aside>;
}
"#;

    let res = AstParser::parse_source_tsx(code).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);
    assert!(!res.has_syntax_errors);

    let names: Vec<&str> = res.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"HeaderBanner"));
    assert!(names.contains(&"Sidebar"));
    assert!(names.contains(&"HeaderProps"));

    let banner = res
        .symbols
        .iter()
        .find(|s| s.name == "HeaderBanner")
        .unwrap();
    assert_eq!(banner.kind, SymbolKind::Arrow);
    assert_eq!(banner.visibility, "export");

    let sidebar = res.symbols.iter().find(|s| s.name == "Sidebar").unwrap();
    assert_eq!(sidebar.kind, SymbolKind::Function);
    assert_eq!(sidebar.visibility, "export");
}

/// 6. test_javascript_commonjs:
/// Extract `module.exports` and `require` in JavaScript
#[test]
fn test_javascript_commonjs() {
    let code = r#"
const path = require('path');
const fs = require('fs');

function readFile(p) {
    return fs.readFileSync(p, 'utf8');
}

const formatPath = (p) => path.resolve(p);

module.exports = {
    readFile,
    formatPath
};
"#;

    let res = AstParser::parse_source(code, Language::JavaScript).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    assert!(res.imports_raw.contains(&"path".to_string()));
    assert!(res.imports_raw.contains(&"fs".to_string()));

    let names: Vec<&str> = res.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"readFile"));
    assert!(names.contains(&"formatPath"));

    // CommonJS exports
    assert!(res.exports.contains(&"readFile".to_string()));
    assert!(res.exports.contains(&"formatPath".to_string()));
}

/// 7. test_javascript_esm:
/// Extract ES6 import and export statements in JavaScript
#[test]
fn test_javascript_esm() {
    let code = r#"
import express from 'express';
import { logger } from './utils';

export const PORT = 8080;

export function startServer() {
    const app = express();
    app.listen(PORT);
}

export default startServer;
"#;

    let res = AstParser::parse_source(code, Language::JavaScript).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    assert!(res.imports_raw.contains(&"express".to_string()));
    assert!(res.imports_raw.contains(&"./utils".to_string()));

    let port = res.symbols.iter().find(|s| s.name == "PORT").unwrap();
    assert_eq!(port.kind, SymbolKind::Constant);
    assert_eq!(port.visibility, "export");

    let start = res
        .symbols
        .iter()
        .find(|s| s.name == "startServer")
        .unwrap();
    assert_eq!(start.kind, SymbolKind::Function);
    assert_eq!(start.visibility, "export");
}

/// 8. test_python_functions:
/// Extract `def` declarations in Python
#[test]
fn test_python_functions() {
    let code = r#"
def public_sum(a: int, b: int) -> int:
    """Return the sum of a and b."""
    return a + b

def _internal_cache_key(key: str) -> str:
    return f"cache_{key}"
"#;

    let res = AstParser::parse_source(code, Language::Python).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);
    assert!(!res.has_syntax_errors);

    let pub_fn = res.symbols.iter().find(|s| s.name == "public_sum").unwrap();
    assert_eq!(pub_fn.kind, SymbolKind::Function);
    assert_eq!(pub_fn.visibility, "public");

    let priv_fn = res
        .symbols
        .iter()
        .find(|s| s.name == "_internal_cache_key")
        .unwrap();
    assert_eq!(priv_fn.kind, SymbolKind::Function);
    assert_eq!(priv_fn.visibility, "private");
}

/// 9. test_python_classes:
/// Extract `class` with methods in Python
#[test]
fn test_python_classes() {
    let code = r#"
class DatabaseSession:
    def __init__(self, uri: str):
        self.uri = uri

    def connect(self) -> bool:
        return True

    def _execute_raw(self, sql: str):
        pass
"#;

    let res = AstParser::parse_source(code, Language::Python).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    let cls = res
        .symbols
        .iter()
        .find(|s| s.name == "DatabaseSession")
        .unwrap();
    assert_eq!(cls.kind, SymbolKind::ClassPython);
    assert_eq!(cls.visibility, "public");

    let connect = res.symbols.iter().find(|s| s.name == "connect").unwrap();
    assert_eq!(connect.kind, SymbolKind::Method);
    assert_eq!(connect.parent.as_deref(), Some("DatabaseSession"));
    assert_eq!(connect.visibility, "public");

    let raw = res
        .symbols
        .iter()
        .find(|s| s.name == "_execute_raw")
        .unwrap();
    assert_eq!(raw.kind, SymbolKind::Method);
    assert_eq!(raw.visibility, "private");
}

/// 10. test_python_decorators:
/// Extract decorated functions in Python
#[test]
fn test_python_decorators() {
    let code = r#"
@app.route('/api/v1/users')
def get_users():
    return []

@dataclass
class UserDto:
    name: str
"#;

    let res = AstParser::parse_source(code, Language::Python).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    let names: Vec<&str> = res.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"get_users"));
    assert!(names.contains(&"UserDto"));

    let dec = res
        .symbols
        .iter()
        .find(|s| s.kind == SymbolKind::Decorator)
        .unwrap();
    assert!(dec.name == "app.route" || dec.name == "dataclass");
}

/// 11. test_python_imports:
/// Extract `import` and `from ... import` in Python
#[test]
fn test_python_imports() {
    let code = r#"
import os
import sys, json
from pathlib import Path
from .models import UserRecord
from ..utils import sanitize_input
"#;

    let res = AstParser::parse_source(code, Language::Python).unwrap();
    assert_eq!(res.parse_status, ParseStatus::Ok);

    assert!(res.imports_raw.contains(&"os".to_string()));
    assert!(res.imports_raw.contains(&"pathlib".to_string()));
    assert!(
        res.imports_raw.contains(&".models".to_string())
            || res.imports_raw.contains(&"..utils".to_string())
    );

    assert!(res.imports.contains(&"os".to_string()));
    assert!(res.imports.contains(&"Path".to_string()));
    assert!(res.imports.contains(&"UserRecord".to_string()));
}

/// 12. test_partial_parse_ts:
/// Parse TypeScript with syntax errors -> parse_status = 'partial' and recovers valid symbols
#[test]
fn test_partial_parse_ts() {
    let fixture_path = Path::new("tests/fixtures/partial_error.ts");
    let res = AstParser::parse_file(fixture_path, Language::TypeScript).unwrap();

    assert!(
        res.has_syntax_errors,
        "Must detect syntax error in partial_error.ts"
    );
    assert_eq!(res.parse_status, ParseStatus::Partial);

    let names: Vec<&str> = res.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"validBeforeError"),
        "Must recover valid function before error"
    );
    assert!(
        names.contains(&"SurvivingClass"),
        "Must recover class after error"
    );
}

/// 13. test_partial_parse_py:
/// Parse Python with syntax errors -> parse_status = 'partial' and recovers valid symbols
#[test]
fn test_partial_parse_py() {
    let fixture_path = Path::new("tests/fixtures/partial_error.py");
    let res = AstParser::parse_file(fixture_path, Language::Python).unwrap();

    assert!(
        res.has_syntax_errors,
        "Must detect syntax error in partial_error.py"
    );
    assert_eq!(res.parse_status, ParseStatus::Partial);

    let names: Vec<&str> = res.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"valid_first"),
        "Must recover valid_first function"
    );
    assert!(
        names.contains(&"SurvivingPyClass"),
        "Must recover SurvivingPyClass"
    );
}

/// 14. test_visibility_ts:
/// Exported vs non-exported symbols in TypeScript
#[test]
fn test_visibility_ts() {
    let code = r#"
export function publicApi() {}
function privateWork() {}
export const PUBLIC_CONFIG = 1;
const LOCAL_CONFIG = 0;
"#;

    let res = AstParser::parse_source(code, Language::TypeScript).unwrap();

    let pub_api = res.symbols.iter().find(|s| s.name == "publicApi").unwrap();
    assert_eq!(pub_api.visibility, "export");

    let priv_work = res
        .symbols
        .iter()
        .find(|s| s.name == "privateWork")
        .unwrap();
    assert_eq!(priv_work.visibility, "private");

    let pub_cfg = res
        .symbols
        .iter()
        .find(|s| s.name == "PUBLIC_CONFIG")
        .unwrap();
    assert_eq!(pub_cfg.visibility, "export");

    let loc_cfg = res
        .symbols
        .iter()
        .find(|s| s.name == "LOCAL_CONFIG")
        .unwrap();
    assert_eq!(loc_cfg.visibility, "private");

    assert!(res.exports.contains(&"publicApi".to_string()));
    assert!(res.exports.contains(&"PUBLIC_CONFIG".to_string()));
    assert!(!res.exports.contains(&"privateWork".to_string()));
}

/// 15. test_visibility_py:
/// `_private` vs public naming convention in Python
#[test]
fn test_visibility_py() {
    let code = r#"
def public_handler():
    pass

def _private_handler():
    pass

class PublicModel:
    pass

class _PrivateModel:
    pass
"#;

    let res = AstParser::parse_source(code, Language::Python).unwrap();

    let pub_h = res
        .symbols
        .iter()
        .find(|s| s.name == "public_handler")
        .unwrap();
    assert_eq!(pub_h.visibility, "public");

    let priv_h = res
        .symbols
        .iter()
        .find(|s| s.name == "_private_handler")
        .unwrap();
    assert_eq!(priv_h.visibility, "private");

    let pub_m = res
        .symbols
        .iter()
        .find(|s| s.name == "PublicModel")
        .unwrap();
    assert_eq!(pub_m.visibility, "public");

    let priv_m = res
        .symbols
        .iter()
        .find(|s| s.name == "_PrivateModel")
        .unwrap();
    assert_eq!(priv_m.visibility, "private");
}

/// 16. test_fixtures_end_to_end:
/// Parse all created sample fixture files from disk
#[test]
fn test_fixtures_end_to_end() {
    let ts_res =
        AstParser::parse_file(Path::new("tests/fixtures/sample.ts"), Language::TypeScript).unwrap();
    assert_eq!(ts_res.parse_status, ParseStatus::Ok);
    assert!(ts_res.symbols.len() >= 6);

    let tsx_res =
        AstParser::parse_file(Path::new("tests/fixtures/sample.tsx"), Language::TypeScript)
            .unwrap();
    assert_eq!(tsx_res.parse_status, ParseStatus::Ok);
    assert!(tsx_res.symbols.len() >= 2);

    let js_res =
        AstParser::parse_file(Path::new("tests/fixtures/sample.js"), Language::JavaScript).unwrap();
    assert_eq!(js_res.parse_status, ParseStatus::Ok);
    assert!(js_res.symbols.len() >= 3);

    let jsx_res =
        AstParser::parse_file(Path::new("tests/fixtures/sample.jsx"), Language::JavaScript)
            .unwrap();
    assert_eq!(jsx_res.parse_status, ParseStatus::Ok);
    assert!(jsx_res.symbols.len() >= 2);

    let py_res =
        AstParser::parse_file(Path::new("tests/fixtures/sample.py"), Language::Python).unwrap();
    assert_eq!(py_res.parse_status, ParseStatus::Ok);
    assert!(py_res.symbols.len() >= 6);
}
