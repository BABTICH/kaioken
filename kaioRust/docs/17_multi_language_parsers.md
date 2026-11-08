# 17. Multi-Language AST Parsers — TypeScript, Python, JavaScript

> **Status**: 🟢 Implemented (Phase 22)
> **Source Module**: `src/scanner/tree_sitter.rs`
> **Affected Modules**: `src/scanner/file_inventory.rs`, `src/watcher/sync.rs`, `Cargo.toml`

---

## 1. Purpose & Motivation

The current kaioken scanner extracts AST symbols **only from Rust** files using `tree-sitter-rust`. To support polyglot repositories (the majority of real-world projects), the scanner must be extended with parsers for:

- **TypeScript / TSX** — Frontend frameworks (React, Next.js, Angular)
- **JavaScript / JSX** — Node.js backends, scripts, legacy codebases
- **Python** — Data science, ML pipelines, backend services

The `Language` enum in [`languages.rs`](file:///d:/project/ai_now_know/kaioRust/src/scanner/languages.rs) already detects these languages and `is_ast_supported()` already returns `true` for them. The missing piece is the **actual tree-sitter grammar integration and symbol extraction logic** for each language.

---

## 2. Current State Assessment

### What Exists

| Component | Status |
|-----------|--------|
| `Language` enum | ✅ Includes Rust, TypeScript, JavaScript, Python + 8 others |
| `Language::detect_from_path()` | ✅ Detects `.ts`, `.tsx`, `.js`, `.jsx`, `.py` extensions |
| `Language::is_ast_supported()` | ✅ Returns `true` for Rust, TS, JS, Python |
| `extract_symbols()` | ⚠️ Only handles `Language::Rust` — others return empty |
| `SymbolKind` enum | ⚠️ Rust-centric (Function, Struct, Enum, Trait, Impl, etc.) |
| `tree-sitter-rust` crate | ✅ In `Cargo.toml` |
| `tree-sitter-typescript` crate | ❌ Not yet added |
| `tree-sitter-python` crate | ❌ Not yet added |

### What's Needed

```text
1. Add tree-sitter grammar crates to Cargo.toml
2. Extend SymbolKind with language-agnostic variants
3. Implement language-specific AST walkers
4. Register parsers in a language→parser dispatch table
5. Add comprehensive tests per language
```

---

## 3. New Dependencies

```toml
# In Cargo.toml [dependencies]
tree-sitter-typescript = "0.23"    # Provides both TypeScript and TSX grammars
tree-sitter-javascript = "0.23"    # JavaScript grammar
tree-sitter-python = "0.23"        # Python grammar
```

---

## 4. Extended `SymbolKind` Enum

The current `SymbolKind` is Rust-focused. We need language-agnostic variants that map naturally across languages:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    // === Universal ===
    Function,       // Rust fn, TS/JS function, Python def
    Method,         // Rust impl fn, TS/JS class method, Python class method
    Struct,         // Rust struct, (conceptual in others)
    Enum,           // Rust enum, TS enum
    Module,         // Rust mod, TS/JS ES module (file-level)
    Constant,       // Rust const, TS/JS const, Python UPPER_CASE
    Static,         // Rust static
    TypeAlias,      // Rust type, TS type alias

    // === Rust-specific ===
    Trait,          // Rust trait
    Impl,           // Rust impl block

    // === TypeScript / JavaScript ===
    Class,          // TS/JS class declaration
    Interface,      // TS interface
    Arrow,          // Arrow function expressions (const fn = () => ...)
    Export,         // Named/default export

    // === Python ===
    Decorator,      // Python @decorator
    ClassPython,    // Python class (distinct node name to avoid collision)
}
```

### Language → SymbolKind Mapping

| Symbol Concept | Rust | TypeScript | JavaScript | Python |
|----------------|------|------------|------------|--------|
| Top-level function | `Function` | `Function` | `Function` | `Function` |
| Class | — | `Class` | `Class` | `ClassPython` |
| Method | `Method` | `Method` | `Method` | `Method` |
| Interface/Trait | `Trait` | `Interface` | — | — |
| Enum | `Enum` | `Enum` | — | — |
| Constant | `Constant` | `Constant` | `Constant` | `Constant` |
| Type alias | `TypeAlias` | `TypeAlias` | — | — |
| Struct | `Struct` | — | — | — |
| Decorator | — | — | — | `Decorator` |
| Arrow function | — | `Arrow` | `Arrow` | — |

---

## 5. Language-Specific AST Walkers

### 5.1 Architecture — Parser Dispatch

```rust
pub fn extract_symbols(path: &Path, source: &str, lang: Language) -> Result<AstExtractResult> {
    match lang {
        Language::Rust => extract_rust_symbols(path, source),
        Language::TypeScript => extract_typescript_symbols(path, source, false),
        Language::JavaScript => extract_javascript_symbols(path, source),
        Language::Python => extract_python_symbols(path, source),
        _ => Ok(AstExtractResult::unsupported()),
    }
}

// TSX handled via flag
fn extract_typescript_symbols(path: &Path, source: &str, is_tsx: bool) -> Result<AstExtractResult> {
    let lang = if is_tsx {
        tree_sitter_typescript::LANGUAGE_TSX
    } else {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT
    };
    // ...
}
```

### 5.2 TypeScript/TSX Node Types to Extract

| tree-sitter Node Type | SymbolKind | Notes |
|------------------------|-----------|-------|
| `function_declaration` | `Function` | `function foo() {}` |
| `class_declaration` | `Class` | `class Foo {}` |
| `method_definition` | `Method` | Inside class body |
| `interface_declaration` | `Interface` | `interface Bar {}` |
| `enum_declaration` | `Enum` | `enum Direction {}` |
| `type_alias_declaration` | `TypeAlias` | `type ID = string` |
| `lexical_declaration` (const + arrow) | `Arrow` | `const fn = () => {}` |
| `export_statement` | `Export` | `export { foo }` / `export default` |
| `variable_declarator` (const UPPER) | `Constant` | `const MAX_SIZE = 100` |

### 5.3 JavaScript Node Types to Extract

| tree-sitter Node Type | SymbolKind | Notes |
|------------------------|-----------|-------|
| `function_declaration` | `Function` | Same as TS |
| `class_declaration` | `Class` | Same as TS |
| `method_definition` | `Method` | Same as TS |
| `arrow_function` (via const) | `Arrow` | `const fn = () => {}` |
| `variable_declarator` (const UPPER) | `Constant` | `const MAX = 10` |
| `export_statement` | `Export` | `module.exports`, `export` |

### 5.4 Python Node Types to Extract

| tree-sitter Node Type | SymbolKind | Notes |
|------------------------|-----------|-------|
| `function_definition` | `Function` | `def foo():` |
| `class_definition` | `ClassPython` | `class Foo:` |
| `decorated_definition` | `Decorator` | `@app.route(...)` + the decorated item |
| Method inside class | `Method` | `def __init__(self):` nested in class |
| UPPER_CASE assignment | `Constant` | `MAX_RETRIES = 3` (heuristic) |

---

## 6. Import Extraction

### Per-Language Import Patterns

| Language | Import Statement | `import_raw` Value |
|----------|------------------|--------------------|
| Rust | `use crate::scanner::git;` | `crate::scanner::git` |
| TypeScript | `import { foo } from './bar';` | `./bar` |
| TypeScript | `import React from 'react';` | `react` |
| JavaScript | `const foo = require('./bar');` | `./bar` |
| JavaScript | `import { foo } from './bar';` | `./bar` (ESM) |
| Python | `import os` | `os` |
| Python | `from pathlib import Path` | `pathlib` |
| Python | `from . import utils` | `.utils` (relative) |

---

## 7. Edge Resolution Enhancements

Currently edges are resolved by matching `import_raw` paths to known files. Multi-language adds:

| Challenge | Solution |
|-----------|----------|
| **Relative imports** (`./bar`) | Resolve relative to importing file's directory |
| **Index files** (`import from './components'`) | Check for `index.ts`, `index.js` |
| **Extension-less imports** | Try `.ts`, `.tsx`, `.js`, `.jsx`, `.py` extensions |
| **External packages** | Mark as `external` edge (not resolved within repo) |
| **Python relative** (`from . import`) | Resolve via package directory structure |

---

## 8. Visibility / Export Detection

| Language | Public | Private |
|----------|--------|---------|
| Rust | `pub`, `pub(crate)`, `pub(super)` | No `pub` keyword |
| TypeScript | `export`, `export default` | No `export` keyword |
| JavaScript | `export`, `module.exports` | Not exported |
| Python | No `_` prefix (convention) | `_private`, `__dunder` |

---

## 9. Testing Strategy

### Test Files

Create test fixtures in `tests/fixtures/`:

```text
tests/fixtures/
├── sample.ts        # TypeScript with classes, interfaces, functions
├── sample.tsx       # TSX with React components
├── sample.js        # JavaScript with CommonJS + ESM
├── sample.jsx       # JSX with React components
├── sample.py        # Python with classes, decorators, imports
├── partial_error.ts # TS file with syntax errors (recovery test)
└── partial_error.py # Python file with syntax errors
```

### Test Cases

| Test | Description |
|------|-------------|
| `test_typescript_functions` | Extract `function` and `arrow` declarations |
| `test_typescript_classes` | Extract class, methods, and interfaces |
| `test_typescript_enums` | Extract enum declarations |
| `test_typescript_imports` | Extract ES6 import statements |
| `test_tsx_components` | Extract React component declarations |
| `test_javascript_commonjs` | Extract `module.exports` and `require` |
| `test_javascript_esm` | Extract ES6 import/export |
| `test_python_functions` | Extract `def` declarations |
| `test_python_classes` | Extract `class` with methods |
| `test_python_decorators` | Extract decorated functions |
| `test_python_imports` | Extract `import` and `from...import` |
| `test_partial_parse_ts` | Parse TS with syntax errors → `parse_status = 'partial'` |
| `test_partial_parse_py` | Parse Python with syntax errors → `parse_status = 'partial'` |
| `test_visibility_ts` | Exported vs non-exported symbols |
| `test_visibility_py` | `_private` vs public naming convention |

---

## 10. Migration / Compatibility

- **Non-breaking**: Existing Rust parsing is unchanged
- **Schema compatible**: The `symbols` table's `kind` column accepts any `SymbolKind` string
- **Incremental rollout**: New parsers can be feature-gated initially

---

## 11. Open Questions

- [ ] Should TSX/JSX component props be extracted as symbol metadata?
- [ ] Should Python type hints (`def foo(x: int) -> str:`) be stored?
- [ ] Should we support **Go** (`tree-sitter-go`) as a fourth language?
- [ ] Should tree-sitter grammars be optional features to reduce binary size?
- [ ] Should `SymbolKind::Class` and `SymbolKind::ClassPython` be unified?
