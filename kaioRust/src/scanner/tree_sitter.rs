use super::file_inventory::ParseStatus;
use super::languages::Language;
use crate::error::{KaiokenError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tree_sitter::{Language as TsLanguage, Node, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    // Universal
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    TypeAlias,
    Module,
    Constant,
    Static,

    // TypeScript / JavaScript
    Class,
    Interface,
    Arrow,
    Export,

    // Python
    Decorator,
    ClassPython,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Impl => "impl",
            SymbolKind::TypeAlias => "type_alias",
            SymbolKind::Module => "module",
            SymbolKind::Constant => "constant",
            SymbolKind::Static => "static",
            SymbolKind::Class => "class",
            SymbolKind::Interface => "interface",
            SymbolKind::Arrow => "arrow",
            SymbolKind::Export => "export",
            SymbolKind::Decorator => "decorator",
            SymbolKind::ClassPython => "class_python",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_start: usize,
    pub line_end: usize,
    pub visibility: String,
    pub signature: Option<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstAnalysisResult {
    pub language: Language,
    pub parse_status: ParseStatus,
    pub symbols: Vec<ExtractedSymbol>,
    pub imports_raw: Vec<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub has_syntax_errors: bool,
}

pub struct AstParser;

impl AstParser {
    pub fn parse_file(path: &Path, language: Language) -> Result<AstAnalysisResult> {
        let content = fs::read_to_string(path)?;
        let is_tsx = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("tsx"))
            .unwrap_or(false);
        Self::parse_source_with_options(&content, language, is_tsx)
    }

    pub fn parse_source(source: &str, language: Language) -> Result<AstAnalysisResult> {
        let is_tsx =
            language == Language::TypeScript && (source.contains("</") || source.contains("/>"));
        Self::parse_source_with_options(source, language, is_tsx)
    }

    pub fn parse_source_tsx(source: &str) -> Result<AstAnalysisResult> {
        Self::parse_source_with_options(source, Language::TypeScript, true)
    }

    pub fn parse_source_with_options(
        source: &str,
        language: Language,
        is_tsx: bool,
    ) -> Result<AstAnalysisResult> {
        match language {
            Language::Rust => Self::parse_rust(source),
            Language::TypeScript => Self::parse_typescript(source, is_tsx),
            Language::JavaScript => Self::parse_javascript(source),
            Language::Python => Self::parse_python(source),
            _ => {
                // Unsupported language for AST: return clean empty result
                Ok(AstAnalysisResult {
                    language,
                    parse_status: ParseStatus::Ok,
                    symbols: Vec::new(),
                    imports_raw: Vec::new(),
                    imports: Vec::new(),
                    exports: Vec::new(),
                    has_syntax_errors: false,
                })
            }
        }
    }

    // =========================================================================
    // 1. Rust Parser
    // =========================================================================

    fn parse_rust(source: &str) -> Result<AstAnalysisResult> {
        let mut parser = Parser::new();
        let lang = tree_sitter_rust::LANGUAGE;
        parser.set_language(&lang.into()).map_err(|e| {
            KaiokenError::ScannerError(format!("Failed to set Rust language: {}", e))
        })?;

        let tree = parser.parse(source, None).ok_or_else(|| {
            KaiokenError::ScannerError("Tree-sitter failed to parse source".to_string())
        })?;

        let root_node = tree.root_node();
        let has_syntax_errors = root_node.has_error();
        let parse_status = if has_syntax_errors {
            ParseStatus::Partial
        } else {
            ParseStatus::Ok
        };

        let mut symbols = Vec::new();
        let mut imports_raw = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        let source_bytes = source.as_bytes();
        Self::walk_rust_nodes(
            root_node,
            source_bytes,
            None,
            &mut symbols,
            &mut imports_raw,
            &mut imports,
            &mut exports,
        );

        Ok(AstAnalysisResult {
            language: Language::Rust,
            parse_status,
            symbols,
            imports_raw,
            imports,
            exports,
            has_syntax_errors,
        })
    }

    fn walk_rust_nodes(
        node: Node,
        source: &[u8],
        parent_name: Option<&str>,
        symbols: &mut Vec<ExtractedSymbol>,
        imports_raw: &mut Vec<String>,
        imports: &mut Vec<String>,
        exports: &mut Vec<String>,
    ) {
        let kind = node.kind();

        match kind {
            "use_declaration" => {
                if let Ok(raw_text) = node.utf8_text(source) {
                    let cleaned = raw_text.trim().to_string();
                    if !imports_raw.contains(&cleaned) {
                        imports_raw.push(cleaned.clone());
                    }

                    let without_use = cleaned
                        .trim_start_matches("pub ")
                        .trim_start_matches("use ")
                        .trim_end_matches(';')
                        .trim();
                    if !without_use.is_empty() && !imports.contains(&without_use.to_string()) {
                        imports.push(without_use.to_string());
                    }
                }
            }
            "function_item" | "function_signature_item" => {
                if let Some(sym) = Self::extract_function_or_method(node, source, parent_name) {
                    if sym.visibility.starts_with("pub") {
                        exports.push(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "struct_item" => {
                if let Some(sym) = Self::extract_named_item(node, source, SymbolKind::Struct) {
                    if sym.visibility.starts_with("pub") {
                        exports.push(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "enum_item" => {
                if let Some(sym) = Self::extract_named_item(node, source, SymbolKind::Enum) {
                    if sym.visibility.starts_with("pub") {
                        exports.push(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "trait_item" => {
                if let Some(sym) = Self::extract_named_item(node, source, SymbolKind::Trait) {
                    if sym.visibility.starts_with("pub") {
                        exports.push(sym.name.clone());
                    }
                    let trait_name = sym.name.clone();
                    symbols.push(sym);

                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "declaration_list" || child.kind().contains("list") {
                            let mut inner_cursor = child.walk();
                            for inner in child.children(&mut inner_cursor) {
                                Self::walk_rust_nodes(
                                    inner,
                                    source,
                                    Some(&trait_name),
                                    symbols,
                                    imports_raw,
                                    imports,
                                    exports,
                                );
                            }
                        }
                    }
                }
            }
            "impl_item" => {
                let impl_name = Self::extract_impl_name(node, source);
                let line_start = node.start_position().row + 1;
                let line_end = node.end_position().row + 1;
                let signature = Self::extract_signature_header(node, source);

                symbols.push(ExtractedSymbol {
                    name: impl_name.clone(),
                    kind: SymbolKind::Impl,
                    line_start,
                    line_end,
                    visibility: "private".to_string(),
                    signature: Some(signature),
                    parent: None,
                });

                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        Self::walk_rust_nodes(
                            child,
                            source,
                            Some(&impl_name),
                            symbols,
                            imports_raw,
                            imports,
                            exports,
                        );
                    }
                }
            }
            "type_item" => {
                if let Some(sym) = Self::extract_named_item(node, source, SymbolKind::TypeAlias) {
                    if sym.visibility.starts_with("pub") {
                        exports.push(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "mod_item" => {
                if let Some(sym) = Self::extract_named_item(node, source, SymbolKind::Module) {
                    if sym.visibility.starts_with("pub") {
                        exports.push(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "const_item" => {
                if let Some(sym) = Self::extract_named_item(node, source, SymbolKind::Constant) {
                    if sym.visibility.starts_with("pub") {
                        exports.push(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            "static_item" => {
                if let Some(sym) = Self::extract_named_item(node, source, SymbolKind::Static) {
                    if sym.visibility.starts_with("pub") {
                        exports.push(sym.name.clone());
                    }
                    symbols.push(sym);
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::walk_rust_nodes(
                        child,
                        source,
                        parent_name,
                        symbols,
                        imports_raw,
                        imports,
                        exports,
                    );
                }
            }
        }
    }

    // =========================================================================
    // 2. TypeScript & TSX Parser
    // =========================================================================

    fn parse_typescript(source: &str, is_tsx: bool) -> Result<AstAnalysisResult> {
        if is_tsx {
            Self::parse_ts_with_lang(
                source,
                tree_sitter_typescript::LANGUAGE_TSX.into(),
                Language::TypeScript,
            )
        } else {
            let res = Self::parse_ts_with_lang(
                source,
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                Language::TypeScript,
            )?;
            if res.has_syntax_errors && (source.contains("</") || source.contains("/>")) {
                if let Ok(tsx_res) = Self::parse_ts_with_lang(
                    source,
                    tree_sitter_typescript::LANGUAGE_TSX.into(),
                    Language::TypeScript,
                ) {
                    if !tsx_res.has_syntax_errors || tsx_res.symbols.len() >= res.symbols.len() {
                        return Ok(tsx_res);
                    }
                }
            }
            Ok(res)
        }
    }

    fn parse_ts_with_lang(
        source: &str,
        ts_lang: TsLanguage,
        language: Language,
    ) -> Result<AstAnalysisResult> {
        let mut parser = Parser::new();
        parser.set_language(&ts_lang).map_err(|e| {
            KaiokenError::ScannerError(format!("Failed to set TypeScript language: {}", e))
        })?;

        let tree = parser.parse(source, None).ok_or_else(|| {
            KaiokenError::ScannerError("Tree-sitter failed to parse TypeScript source".to_string())
        })?;

        let root_node = tree.root_node();
        let has_syntax_errors = root_node.has_error();
        let parse_status = if has_syntax_errors {
            ParseStatus::Partial
        } else {
            ParseStatus::Ok
        };

        let mut symbols = Vec::new();
        let mut imports_raw = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        let source_bytes = source.as_bytes();
        Self::walk_ts_nodes(
            root_node,
            source_bytes,
            None,
            false,
            &mut symbols,
            &mut imports_raw,
            &mut imports,
            &mut exports,
        );

        Ok(AstAnalysisResult {
            language,
            parse_status,
            symbols,
            imports_raw,
            imports,
            exports,
            has_syntax_errors,
        })
    }

    fn walk_ts_nodes(
        node: Node,
        source: &[u8],
        parent_name: Option<&str>,
        is_exported: bool,
        symbols: &mut Vec<ExtractedSymbol>,
        imports_raw: &mut Vec<String>,
        imports: &mut Vec<String>,
        exports: &mut Vec<String>,
    ) {
        let kind = node.kind();

        match kind {
            "import_statement" => {
                if let Some(source_node) = node.child_by_field_name("source") {
                    if let Ok(raw_text) = source_node.utf8_text(source) {
                        let cleaned = raw_text
                            .trim_matches(|c| c == '\'' || c == '"' || c == '`')
                            .to_string();
                        if !cleaned.is_empty() && !imports_raw.contains(&cleaned) {
                            imports_raw.push(cleaned);
                        }
                    }
                }

                // Extract imported identifiers
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "import_clause" {
                        let mut ic_cursor = child.walk();
                        for ic_child in child.children(&mut ic_cursor) {
                            if ic_child.kind() == "identifier" {
                                if let Ok(name) = ic_child.utf8_text(source) {
                                    if !imports.contains(&name.to_string()) {
                                        imports.push(name.to_string());
                                    }
                                }
                            } else if ic_child.kind() == "named_imports" {
                                let mut ni_cursor = ic_child.walk();
                                for spec in ic_child.children(&mut ni_cursor) {
                                    if spec.kind() == "import_specifier" {
                                        let target =
                                            spec.child_by_field_name("name").unwrap_or(spec);
                                        if let Ok(name) = target.utf8_text(source) {
                                            if !imports.contains(&name.to_string()) {
                                                imports.push(name.to_string());
                                            }
                                        }
                                    }
                                }
                            } else if ic_child.kind() == "namespace_import" {
                                let target =
                                    ic_child.child_by_field_name("name").unwrap_or(ic_child);
                                if let Ok(name) = target.utf8_text(source) {
                                    let clean_name = name.trim_start_matches("* as ").trim();
                                    if !imports.contains(&clean_name.to_string()) {
                                        imports.push(clean_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "export_statement" => {
                // If this is `export default ...` or `export { foo, bar }` or `export const ...`
                if let Some(decl) = node.child_by_field_name("declaration") {
                    Self::walk_ts_nodes(
                        decl,
                        source,
                        parent_name,
                        true,
                        symbols,
                        imports_raw,
                        imports,
                        exports,
                    );
                    return;
                }

                // Check for export clause specifiers: export { a, b }
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "export_clause" {
                        let mut ec_cursor = child.walk();
                        for spec in child.children(&mut ec_cursor) {
                            if spec.kind() == "export_specifier" {
                                let target = spec.child_by_field_name("name").unwrap_or(spec);
                                if let Ok(name) = target.utf8_text(source) {
                                    let clean_name = name.trim().to_string();
                                    if !exports.contains(&clean_name) {
                                        exports.push(clean_name.clone());
                                    }
                                    symbols.push(ExtractedSymbol {
                                        name: clean_name,
                                        kind: SymbolKind::Export,
                                        line_start: spec.start_position().row + 1,
                                        line_end: spec.end_position().row + 1,
                                        visibility: "export".to_string(),
                                        signature: Some(
                                            spec.utf8_text(source).unwrap_or("").to_string(),
                                        ),
                                        parent: None,
                                    });
                                }
                            }
                        }
                    } else if child.kind() != "export" && child.kind() != "default" {
                        Self::walk_ts_nodes(
                            child,
                            source,
                            parent_name,
                            true,
                            symbols,
                            imports_raw,
                            imports,
                            exports,
                        );
                    }
                }
            }
            "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if is_exported {
                            "export".to_string()
                        } else {
                            "private".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);

                        if is_exported && !exports.contains(&name.to_string()) {
                            exports.push(name.to_string());
                        }

                        symbols.push(ExtractedSymbol {
                            name: name.to_string(),
                            kind: SymbolKind::Function,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: parent_name.map(str::to_string),
                        });
                    }
                }
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let class_name = name.to_string();
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if is_exported {
                            "export".to_string()
                        } else {
                            "private".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);

                        if is_exported && !exports.contains(&class_name) {
                            exports.push(class_name.clone());
                        }

                        symbols.push(ExtractedSymbol {
                            name: class_name.clone(),
                            kind: SymbolKind::Class,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: None,
                        });

                        // Walk class body
                        if let Some(body) = node.child_by_field_name("body") {
                            let mut cursor = body.walk();
                            for child in body.children(&mut cursor) {
                                if child.kind() == "method_definition" {
                                    if let Some(m_name_node) = child.child_by_field_name("name") {
                                        if let Ok(m_name) = m_name_node.utf8_text(source) {
                                            let m_start = child.start_position().row + 1;
                                            let m_end = child.end_position().row + 1;
                                            let m_vis =
                                                Self::extract_ts_method_visibility(child, source);
                                            let m_sig =
                                                Self::extract_signature_header(child, source);

                                            symbols.push(ExtractedSymbol {
                                                name: m_name.to_string(),
                                                kind: SymbolKind::Method,
                                                line_start: m_start,
                                                line_end: m_end,
                                                visibility: m_vis,
                                                signature: Some(m_sig),
                                                parent: Some(class_name.clone()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "interface_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if is_exported {
                            "export".to_string()
                        } else {
                            "private".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);

                        if is_exported && !exports.contains(&name.to_string()) {
                            exports.push(name.to_string());
                        }

                        symbols.push(ExtractedSymbol {
                            name: name.to_string(),
                            kind: SymbolKind::Interface,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: None,
                        });
                    }
                }
            }
            "enum_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if is_exported {
                            "export".to_string()
                        } else {
                            "private".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);

                        if is_exported && !exports.contains(&name.to_string()) {
                            exports.push(name.to_string());
                        }

                        symbols.push(ExtractedSymbol {
                            name: name.to_string(),
                            kind: SymbolKind::Enum,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: None,
                        });
                    }
                }
            }
            "type_alias_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if is_exported {
                            "export".to_string()
                        } else {
                            "private".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);

                        if is_exported && !exports.contains(&name.to_string()) {
                            exports.push(name.to_string());
                        }

                        symbols.push(ExtractedSymbol {
                            name: name.to_string(),
                            kind: SymbolKind::TypeAlias,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: None,
                        });
                    }
                }
            }
            "lexical_declaration" | "variable_declaration" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "variable_declarator" {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source) {
                                let line_start = child.start_position().row + 1;
                                let line_end = child.end_position().row + 1;
                                let visibility = if is_exported {
                                    "export".to_string()
                                } else {
                                    "private".to_string()
                                };
                                let signature = Self::extract_signature_header(child, source);

                                let value_opt = child.child_by_field_name("value");
                                let is_arrow = value_opt
                                    .map(|v| {
                                        v.kind() == "arrow_function"
                                            || v.kind() == "function_expression"
                                    })
                                    .unwrap_or(false);
                                let is_const = name.chars().all(|c| {
                                    c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()
                                }) && name.len() >= 2;

                                if is_arrow {
                                    if is_exported && !exports.contains(&name.to_string()) {
                                        exports.push(name.to_string());
                                    }
                                    symbols.push(ExtractedSymbol {
                                        name: name.to_string(),
                                        kind: SymbolKind::Arrow,
                                        line_start,
                                        line_end,
                                        visibility,
                                        signature: Some(signature),
                                        parent: None,
                                    });
                                } else if is_const {
                                    if is_exported && !exports.contains(&name.to_string()) {
                                        exports.push(name.to_string());
                                    }
                                    symbols.push(ExtractedSymbol {
                                        name: name.to_string(),
                                        kind: SymbolKind::Constant,
                                        line_start,
                                        line_end,
                                        visibility,
                                        signature: Some(signature),
                                        parent: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::walk_ts_nodes(
                        child,
                        source,
                        parent_name,
                        is_exported,
                        symbols,
                        imports_raw,
                        imports,
                        exports,
                    );
                }
            }
        }
    }

    fn extract_ts_method_visibility(node: Node, source: &[u8]) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "accessibility_modifier" {
                if let Ok(text) = child.utf8_text(source) {
                    return text.trim().to_string();
                }
            }
        }
        "public".to_string()
    }

    // =========================================================================
    // 3. JavaScript Parser
    // =========================================================================

    fn parse_javascript(source: &str) -> Result<AstAnalysisResult> {
        let mut parser = Parser::new();
        let lang = tree_sitter_javascript::LANGUAGE;
        parser.set_language(&lang.into()).map_err(|e| {
            KaiokenError::ScannerError(format!("Failed to set JavaScript language: {}", e))
        })?;

        let tree = parser.parse(source, None).ok_or_else(|| {
            KaiokenError::ScannerError("Tree-sitter failed to parse JavaScript source".to_string())
        })?;

        let root_node = tree.root_node();
        let has_syntax_errors = root_node.has_error();
        let parse_status = if has_syntax_errors {
            ParseStatus::Partial
        } else {
            ParseStatus::Ok
        };

        let mut symbols = Vec::new();
        let mut imports_raw = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        let source_bytes = source.as_bytes();
        Self::walk_js_nodes(
            root_node,
            source_bytes,
            None,
            false,
            &mut symbols,
            &mut imports_raw,
            &mut imports,
            &mut exports,
        );

        Ok(AstAnalysisResult {
            language: Language::JavaScript,
            parse_status,
            symbols,
            imports_raw,
            imports,
            exports,
            has_syntax_errors,
        })
    }

    fn walk_js_nodes(
        node: Node,
        source: &[u8],
        parent_name: Option<&str>,
        is_exported: bool,
        symbols: &mut Vec<ExtractedSymbol>,
        imports_raw: &mut Vec<String>,
        imports: &mut Vec<String>,
        exports: &mut Vec<String>,
    ) {
        let kind = node.kind();

        match kind {
            "import_statement" => {
                if let Some(source_node) = node.child_by_field_name("source") {
                    if let Ok(raw_text) = source_node.utf8_text(source) {
                        let cleaned = raw_text
                            .trim_matches(|c| c == '\'' || c == '"' || c == '`')
                            .to_string();
                        if !cleaned.is_empty() && !imports_raw.contains(&cleaned) {
                            imports_raw.push(cleaned);
                        }
                    }
                }

                // Extract imported identifiers
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "import_clause" {
                        let mut ic_cursor = child.walk();
                        for ic_child in child.children(&mut ic_cursor) {
                            if ic_child.kind() == "identifier" {
                                if let Ok(name) = ic_child.utf8_text(source) {
                                    if !imports.contains(&name.to_string()) {
                                        imports.push(name.to_string());
                                    }
                                }
                            } else if ic_child.kind() == "named_imports" {
                                let mut ni_cursor = ic_child.walk();
                                for spec in ic_child.children(&mut ni_cursor) {
                                    if spec.kind() == "import_specifier" {
                                        let target =
                                            spec.child_by_field_name("name").unwrap_or(spec);
                                        if let Ok(name) = target.utf8_text(source) {
                                            if !imports.contains(&name.to_string()) {
                                                imports.push(name.to_string());
                                            }
                                        }
                                    }
                                }
                            } else if ic_child.kind() == "namespace_import" {
                                let target =
                                    ic_child.child_by_field_name("name").unwrap_or(ic_child);
                                if let Ok(name) = target.utf8_text(source) {
                                    let clean_name = name.trim_start_matches("* as ").trim();
                                    if !imports.contains(&clean_name.to_string()) {
                                        imports.push(clean_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "export_statement" => {
                if let Some(decl) = node.child_by_field_name("declaration") {
                    Self::walk_js_nodes(
                        decl,
                        source,
                        parent_name,
                        true,
                        symbols,
                        imports_raw,
                        imports,
                        exports,
                    );
                    return;
                }

                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "export_clause" {
                        let mut ec_cursor = child.walk();
                        for spec in child.children(&mut ec_cursor) {
                            if spec.kind() == "export_specifier" {
                                let target = spec.child_by_field_name("name").unwrap_or(spec);
                                if let Ok(name) = target.utf8_text(source) {
                                    let clean_name = name.trim().to_string();
                                    if !exports.contains(&clean_name) {
                                        exports.push(clean_name.clone());
                                    }
                                    symbols.push(ExtractedSymbol {
                                        name: clean_name,
                                        kind: SymbolKind::Export,
                                        line_start: spec.start_position().row + 1,
                                        line_end: spec.end_position().row + 1,
                                        visibility: "export".to_string(),
                                        signature: Some(
                                            spec.utf8_text(source).unwrap_or("").to_string(),
                                        ),
                                        parent: None,
                                    });
                                }
                            }
                        }
                    } else if child.kind() != "export" && child.kind() != "default" {
                        Self::walk_js_nodes(
                            child,
                            source,
                            parent_name,
                            true,
                            symbols,
                            imports_raw,
                            imports,
                            exports,
                        );
                    }
                }
            }
            "expression_statement" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "call_expression" {
                        Self::check_js_require(child, source, imports_raw, imports);
                    } else if child.kind() == "assignment_expression" {
                        Self::check_js_commonjs_export(child, source, symbols, exports);
                    }
                }
            }
            "lexical_declaration" | "variable_declaration" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "variable_declarator" {
                        let name_opt = child
                            .child_by_field_name("name")
                            .and_then(|n| n.utf8_text(source).ok());
                        let value_opt = child.child_by_field_name("value");

                        // Check const x = require('./x')
                        if let Some(val) = value_opt {
                            if val.kind() == "call_expression" {
                                Self::check_js_require(val, source, imports_raw, imports);
                            }
                        }

                        if let Some(name) = name_opt {
                            let line_start = child.start_position().row + 1;
                            let line_end = child.end_position().row + 1;
                            let visibility = if is_exported {
                                "export".to_string()
                            } else {
                                "private".to_string()
                            };
                            let signature = Self::extract_signature_header(child, source);

                            let is_arrow = value_opt
                                .map(|v| {
                                    v.kind() == "arrow_function"
                                        || v.kind() == "function_expression"
                                })
                                .unwrap_or(false);
                            let is_const = name
                                .chars()
                                .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
                                && name.len() >= 2;

                            if is_arrow {
                                if is_exported && !exports.contains(&name.to_string()) {
                                    exports.push(name.to_string());
                                }
                                symbols.push(ExtractedSymbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Arrow,
                                    line_start,
                                    line_end,
                                    visibility,
                                    signature: Some(signature),
                                    parent: None,
                                });
                            } else if is_const {
                                if is_exported && !exports.contains(&name.to_string()) {
                                    exports.push(name.to_string());
                                }
                                symbols.push(ExtractedSymbol {
                                    name: name.to_string(),
                                    kind: SymbolKind::Constant,
                                    line_start,
                                    line_end,
                                    visibility,
                                    signature: Some(signature),
                                    parent: None,
                                });
                            }
                        }
                    }
                }
            }
            "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if is_exported {
                            "export".to_string()
                        } else {
                            "private".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);

                        if is_exported && !exports.contains(&name.to_string()) {
                            exports.push(name.to_string());
                        }

                        symbols.push(ExtractedSymbol {
                            name: name.to_string(),
                            kind: SymbolKind::Function,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: parent_name.map(str::to_string),
                        });
                    }
                }
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let class_name = name.to_string();
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if is_exported {
                            "export".to_string()
                        } else {
                            "private".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);

                        if is_exported && !exports.contains(&class_name) {
                            exports.push(class_name.clone());
                        }

                        symbols.push(ExtractedSymbol {
                            name: class_name.clone(),
                            kind: SymbolKind::Class,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: None,
                        });

                        if let Some(body) = node.child_by_field_name("body") {
                            let mut cursor = body.walk();
                            for child in body.children(&mut cursor) {
                                if child.kind() == "method_definition" {
                                    if let Some(m_name_node) = child.child_by_field_name("name") {
                                        if let Ok(m_name) = m_name_node.utf8_text(source) {
                                            let m_start = child.start_position().row + 1;
                                            let m_end = child.end_position().row + 1;
                                            let m_sig =
                                                Self::extract_signature_header(child, source);

                                            symbols.push(ExtractedSymbol {
                                                name: m_name.to_string(),
                                                kind: SymbolKind::Method,
                                                line_start: m_start,
                                                line_end: m_end,
                                                visibility: "public".to_string(),
                                                signature: Some(m_sig),
                                                parent: Some(class_name.clone()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::walk_js_nodes(
                        child,
                        source,
                        parent_name,
                        is_exported,
                        symbols,
                        imports_raw,
                        imports,
                        exports,
                    );
                }
            }
        }
    }

    fn check_js_require(
        call_node: Node,
        source: &[u8],
        imports_raw: &mut Vec<String>,
        imports: &mut Vec<String>,
    ) {
        let is_require = if let Some(func_node) = call_node.child_by_field_name("function") {
            func_node
                .utf8_text(source)
                .map(|s| s.trim() == "require")
                .unwrap_or(false)
        } else {
            call_node
                .child(0)
                .and_then(|c| c.utf8_text(source).ok())
                .map(|s| s.trim() == "require")
                .unwrap_or(false)
        };

        if is_require {
            if let Some(args_node) = call_node.child_by_field_name("arguments") {
                let mut cursor = args_node.walk();
                for child in args_node.children(&mut cursor) {
                    if child.kind() == "string" {
                        if let Ok(text) = child.utf8_text(source) {
                            let cleaned = text
                                .trim_matches(|c| c == '\'' || c == '"' || c == '`')
                                .to_string();
                            if !cleaned.is_empty() && !imports_raw.contains(&cleaned) {
                                imports_raw.push(cleaned.clone());
                            }
                            if !cleaned.is_empty() && !imports.contains(&cleaned) {
                                imports.push(cleaned);
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_js_commonjs_export(
        assign_node: Node,
        source: &[u8],
        symbols: &mut Vec<ExtractedSymbol>,
        exports: &mut Vec<String>,
    ) {
        if let Some(left) = assign_node.child_by_field_name("left") {
            if let Ok(left_text) = left.utf8_text(source) {
                let left_str = left_text.trim();
                let line_start = assign_node.start_position().row + 1;
                let line_end = assign_node.end_position().row + 1;

                if left_str == "module.exports" {
                    if let Some(right) = assign_node.child_by_field_name("right") {
                        if right.kind() == "object" {
                            let mut cursor = right.walk();
                            for child in right.children(&mut cursor) {
                                if child.kind() == "shorthand_property_identifier"
                                    || child.kind() == "pair"
                                {
                                    let prop_name = child
                                        .child_by_field_name("key")
                                        .unwrap_or(child)
                                        .utf8_text(source)
                                        .unwrap_or("")
                                        .trim()
                                        .to_string();
                                    if !prop_name.is_empty() && !exports.contains(&prop_name) {
                                        exports.push(prop_name.clone());
                                        symbols.push(ExtractedSymbol {
                                            name: prop_name,
                                            kind: SymbolKind::Export,
                                            line_start,
                                            line_end,
                                            visibility: "export".to_string(),
                                            signature: Some(
                                                child.utf8_text(source).unwrap_or("").to_string(),
                                            ),
                                            parent: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                } else if left_str.starts_with("module.exports.")
                    || left_str.starts_with("exports.")
                {
                    let prop = left_str
                        .strip_prefix("module.exports.")
                        .or_else(|| left_str.strip_prefix("exports."))
                        .unwrap_or_default();

                    let prop_name = prop.trim().to_string();
                    if !prop_name.is_empty() && !exports.contains(&prop_name) {
                        exports.push(prop_name.clone());
                        symbols.push(ExtractedSymbol {
                            name: prop_name,
                            kind: SymbolKind::Export,
                            line_start,
                            line_end,
                            visibility: "export".to_string(),
                            signature: Some(
                                assign_node.utf8_text(source).unwrap_or("").to_string(),
                            ),
                            parent: None,
                        });
                    }
                }
            }
        }
    }

    // =========================================================================
    // 4. Python Parser
    // =========================================================================

    fn parse_python(source: &str) -> Result<AstAnalysisResult> {
        let mut parser = Parser::new();
        let lang = tree_sitter_python::LANGUAGE;
        parser.set_language(&lang.into()).map_err(|e| {
            KaiokenError::ScannerError(format!("Failed to set Python language: {}", e))
        })?;

        let tree = parser.parse(source, None).ok_or_else(|| {
            KaiokenError::ScannerError("Tree-sitter failed to parse Python source".to_string())
        })?;

        let root_node = tree.root_node();
        let has_syntax_errors = root_node.has_error();
        let parse_status = if has_syntax_errors {
            ParseStatus::Partial
        } else {
            ParseStatus::Ok
        };

        let mut symbols = Vec::new();
        let mut imports_raw = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        let source_bytes = source.as_bytes();
        Self::walk_py_nodes(
            root_node,
            source_bytes,
            None,
            &mut symbols,
            &mut imports_raw,
            &mut imports,
            &mut exports,
        );

        Ok(AstAnalysisResult {
            language: Language::Python,
            parse_status,
            symbols,
            imports_raw,
            imports,
            exports,
            has_syntax_errors,
        })
    }

    fn walk_py_nodes(
        node: Node,
        source: &[u8],
        parent_name: Option<&str>,
        symbols: &mut Vec<ExtractedSymbol>,
        imports_raw: &mut Vec<String>,
        imports: &mut Vec<String>,
        exports: &mut Vec<String>,
    ) {
        let kind = node.kind();

        match kind {
            "import_statement" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "dotted_name" {
                        if let Ok(name) = child.utf8_text(source) {
                            let name_str = name.trim().to_string();
                            if !imports_raw.contains(&name_str) {
                                imports_raw.push(name_str.clone());
                            }
                            if !imports.contains(&name_str) {
                                imports.push(name_str);
                            }
                        }
                    } else if child.kind() == "aliased_import" {
                        if let Some(target) = child.child_by_field_name("name") {
                            if let Ok(name) = target.utf8_text(source) {
                                let name_str = name.trim().to_string();
                                if !imports_raw.contains(&name_str) {
                                    imports_raw.push(name_str);
                                }
                            }
                        }
                        if let Some(alias) = child.child_by_field_name("alias") {
                            if let Ok(alias_name) = alias.utf8_text(source) {
                                let alias_str = alias_name.trim().to_string();
                                if !imports.contains(&alias_str) {
                                    imports.push(alias_str);
                                }
                            }
                        }
                    }
                }
            }
            "import_from_statement" => {
                if let Some(module_node) = node.child_by_field_name("module_name") {
                    if let Ok(module_text) = module_node.utf8_text(source) {
                        let mod_str = module_text.trim().to_string();
                        if !imports_raw.contains(&mod_str) {
                            imports_raw.push(mod_str);
                        }
                    }
                } else {
                    // Check for relative import dots (e.g. `from . import foo`)
                    let mut cursor = node.walk();
                    let mut dots = String::new();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "relative_import" || child.kind() == "." {
                            dots.push_str(child.utf8_text(source).unwrap_or("."));
                        }
                    }
                    if !dots.is_empty() && !imports_raw.contains(&dots) {
                        imports_raw.push(dots);
                    }
                }

                // Collect imported names
                let mut cursor = node.walk();
                let mut found_import_kw = false;
                for child in node.children(&mut cursor) {
                    if child.kind() == "import" {
                        found_import_kw = true;
                    } else if found_import_kw {
                        if child.kind() == "dotted_name" || child.kind() == "identifier" {
                            if let Ok(name) = child.utf8_text(source) {
                                let name_str = name.trim().to_string();
                                if !imports.contains(&name_str) {
                                    imports.push(name_str);
                                }
                            }
                        } else if child.kind() == "aliased_import" {
                            let target = child
                                .child_by_field_name("alias")
                                .or_else(|| child.child_by_field_name("name"));
                            if let Some(t) = target {
                                if let Ok(name) = t.utf8_text(source) {
                                    let name_str = name.trim().to_string();
                                    if !imports.contains(&name_str) {
                                        imports.push(name_str);
                                    }
                                }
                            }
                        } else if child.kind() == "wildcard_import"
                            && !imports.contains(&"*".to_string())
                        {
                            imports.push("*".to_string());
                        }
                    }
                }
            }
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let func_name = name.to_string();
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if func_name.starts_with('_') {
                            "private".to_string()
                        } else {
                            "public".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);
                        let kind = if parent_name.is_some() {
                            SymbolKind::Method
                        } else {
                            SymbolKind::Function
                        };

                        if visibility == "public"
                            && parent_name.is_none()
                            && !exports.contains(&func_name)
                        {
                            exports.push(func_name.clone());
                        }

                        symbols.push(ExtractedSymbol {
                            name: func_name,
                            kind,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: parent_name.map(str::to_string),
                        });
                    }
                }
            }
            "class_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source) {
                        let class_name = name.to_string();
                        let line_start = node.start_position().row + 1;
                        let line_end = node.end_position().row + 1;
                        let visibility = if class_name.starts_with('_') {
                            "private".to_string()
                        } else {
                            "public".to_string()
                        };
                        let signature = Self::extract_signature_header(node, source);

                        if visibility == "public" && !exports.contains(&class_name) {
                            exports.push(class_name.clone());
                        }

                        symbols.push(ExtractedSymbol {
                            name: class_name.clone(),
                            kind: SymbolKind::ClassPython,
                            line_start,
                            line_end,
                            visibility,
                            signature: Some(signature),
                            parent: None,
                        });

                        // Walk class body block
                        if let Some(body) = node.child_by_field_name("body") {
                            let mut cursor = body.walk();
                            for child in body.children(&mut cursor) {
                                Self::walk_py_nodes(
                                    child,
                                    source,
                                    Some(&class_name),
                                    symbols,
                                    imports_raw,
                                    imports,
                                    exports,
                                );
                            }
                        }
                    }
                }
            }
            "decorated_definition" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "decorator" {
                        if let Ok(dec_text) = child.utf8_text(source) {
                            let dec_raw = dec_text.trim();
                            let dec_name = dec_raw
                                .trim_start_matches('@')
                                .split('(')
                                .next()
                                .unwrap_or(dec_raw)
                                .trim()
                                .to_string();

                            symbols.push(ExtractedSymbol {
                                name: dec_name,
                                kind: SymbolKind::Decorator,
                                line_start: child.start_position().row + 1,
                                line_end: child.end_position().row + 1,
                                visibility: "public".to_string(),
                                signature: Some(dec_raw.to_string()),
                                parent: None,
                            });
                        }
                    } else if child.kind() == "function_definition"
                        || child.kind() == "class_definition"
                    {
                        Self::walk_py_nodes(
                            child,
                            source,
                            parent_name,
                            symbols,
                            imports_raw,
                            imports,
                            exports,
                        );
                    }
                }
            }
            "expression_statement" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "assignment" {
                        if let Some(left) = child.child_by_field_name("left") {
                            if left.kind() == "identifier" {
                                if let Ok(name) = left.utf8_text(source) {
                                    let name_str = name.trim().to_string();
                                    if name_str.chars().all(|c| {
                                        c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()
                                    }) && name_str.len() >= 2
                                    {
                                        let line_start = child.start_position().row + 1;
                                        let line_end = child.end_position().row + 1;
                                        let visibility = if name_str.starts_with('_') {
                                            "private".to_string()
                                        } else {
                                            "public".to_string()
                                        };
                                        let signature =
                                            Self::extract_signature_header(child, source);

                                        if visibility == "public"
                                            && parent_name.is_none()
                                            && !exports.contains(&name_str)
                                        {
                                            exports.push(name_str.clone());
                                        }

                                        symbols.push(ExtractedSymbol {
                                            name: name_str,
                                            kind: SymbolKind::Constant,
                                            line_start,
                                            line_end,
                                            visibility,
                                            signature: Some(signature),
                                            parent: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::walk_py_nodes(
                        child,
                        source,
                        parent_name,
                        symbols,
                        imports_raw,
                        imports,
                        exports,
                    );
                }
            }
        }
    }

    // =========================================================================
    // Common Helper Functions
    // =========================================================================

    fn extract_function_or_method(
        node: Node,
        source: &[u8],
        parent_name: Option<&str>,
    ) -> Option<ExtractedSymbol> {
        let name_node = node.child_by_field_name("name")?;
        let name = name_node.utf8_text(source).ok()?.to_string();

        let visibility = Self::extract_visibility(node, source);
        let line_start = node.start_position().row + 1;
        let line_end = node.end_position().row + 1;
        let signature = Self::extract_signature_header(node, source);

        let kind = if parent_name.is_some() {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };

        Some(ExtractedSymbol {
            name,
            kind,
            line_start,
            line_end,
            visibility,
            signature: Some(signature),
            parent: parent_name.map(|s| s.to_string()),
        })
    }

    fn extract_named_item(node: Node, source: &[u8], kind: SymbolKind) -> Option<ExtractedSymbol> {
        let name_node = node.child_by_field_name("name")?;
        let name = name_node.utf8_text(source).ok()?.to_string();

        let visibility = Self::extract_visibility(node, source);
        let line_start = node.start_position().row + 1;
        let line_end = node.end_position().row + 1;
        let signature = Self::extract_signature_header(node, source);

        Some(ExtractedSymbol {
            name,
            kind,
            line_start,
            line_end,
            visibility,
            signature: Some(signature),
            parent: None,
        })
    }

    fn extract_visibility(node: Node, source: &[u8]) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                if let Ok(text) = child.utf8_text(source) {
                    return text.trim().to_string();
                }
            }
        }
        "private".to_string()
    }

    fn extract_impl_name(node: Node, source: &[u8]) -> String {
        let type_node = node.child_by_field_name("type");
        let trait_node = node.child_by_field_name("trait");

        match (trait_node, type_node) {
            (Some(tr), Some(ty)) => {
                let tr_text = tr.utf8_text(source).unwrap_or("Trait");
                let ty_text = ty.utf8_text(source).unwrap_or("Type");
                format!("impl {} for {}", tr_text, ty_text)
            }
            (None, Some(ty)) => {
                let ty_text = ty.utf8_text(source).unwrap_or("Type");
                format!("impl {}", ty_text)
            }
            _ => "impl".to_string(),
        }
    }

    fn extract_signature_header(node: Node, source: &[u8]) -> String {
        let full_text = node.utf8_text(source).unwrap_or("");
        if let Some(brace_idx) = full_text.find('{') {
            full_text[..brace_idx].trim().to_string()
        } else if let Some(semi_idx) = full_text.find(';') {
            full_text[..semi_idx].trim().to_string()
        } else if full_text.contains(':') {
            // For python headers, or first line
            let first_line = full_text.lines().next().unwrap_or("").trim();
            if let Some(c_idx) = first_line.find(':') {
                first_line[..c_idx].trim().to_string()
            } else {
                first_line.to_string()
            }
        } else {
            full_text.lines().next().unwrap_or("").trim().to_string()
        }
    }
}
