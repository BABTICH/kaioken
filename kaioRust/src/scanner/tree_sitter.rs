use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser};
use crate::error::{KaiokenError, Result};
use super::languages::Language;
use super::file_inventory::ParseStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
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
        Self::parse_source(&content, language)
    }

    pub fn parse_source(source: &str, language: Language) -> Result<AstAnalysisResult> {
        match language {
            Language::Rust => Self::parse_rust(source),
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

    fn parse_rust(source: &str) -> Result<AstAnalysisResult> {
        let mut parser = Parser::new();
        let lang = tree_sitter_rust::LANGUAGE;
        parser
            .set_language(&lang.into())
            .map_err(|e| KaiokenError::ScannerError(format!("Failed to set Rust language: {}", e)))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| KaiokenError::ScannerError("Tree-sitter failed to parse source".to_string()))?;

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

                    // Extract normalized path (e.g. `use std::path::Path;` -> `std::path::Path`)
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

                    // Recurse into trait body / declaration list to extract method declarations
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

                // Walk methods inside impl body
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
                // For root or other container nodes, recurse through children
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
        // Capture up to opening brace '{' or semicolon ';', or first line
        if let Some(brace_idx) = full_text.find('{') {
            full_text[..brace_idx].trim().to_string()
        } else if let Some(semi_idx) = full_text.find(';') {
            full_text[..semi_idx].trim().to_string()
        } else {
            full_text.lines().next().unwrap_or("").trim().to_string()
        }
    }
}
