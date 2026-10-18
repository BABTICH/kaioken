use std::fs;
use kaioken::scanner::{
    FileInventory, FileInventoryOptions, Language, ParseStatus,
};

#[test]
fn test_inventory_walk_and_language_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    // Create source files
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("src").join("app.ts"), "export const x = 1;\n").unwrap();
    fs::write(root.join("src").join("script.py"), "print('hello')\n").unwrap();
    fs::write(root.join("README.md"), "# Test Project\n").unwrap();

    let summary = FileInventory::scan_dir(root, &[]).unwrap();
    assert_eq!(summary.total_files, 4);

    let rust_file = summary.files.iter().find(|f| f.relative_path.to_str().unwrap().contains("main.rs")).unwrap();
    assert_eq!(rust_file.language, Language::Rust);
    assert_eq!(rust_file.parse_status, ParseStatus::Ok);
    assert!(rust_file.file_hash.is_some());

    let ts_file = summary.files.iter().find(|f| f.relative_path.to_str().unwrap().contains("app.ts")).unwrap();
    assert_eq!(ts_file.language, Language::TypeScript);
    assert_eq!(ts_file.parse_status, ParseStatus::Ok);
}

#[test]
fn test_inventory_sha256_hash_computation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("content.txt");
    fs::write(&file_path, "hello world\n").unwrap();

    let options = FileInventoryOptions::default();
    let discovered = FileInventory::inspect_file(temp_dir.path(), &file_path, &options).unwrap();

    assert!(discovered.file_hash.is_some());
    let hash = discovered.file_hash.unwrap();
    assert_eq!(hash.len(), 64, "SHA-256 hash must be 64 characters");
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_inventory_binary_file_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_path = temp_dir.path().join("image.png");

    // File with null bytes simulating binary payload
    let binary_data = vec![0x89, 0x50, 0x4E, 0x47, 0x00, 0x0D, 0x0A, 0x1A];
    fs::write(&bin_path, binary_data).unwrap();

    let options = FileInventoryOptions::default();
    let discovered = FileInventory::inspect_file(temp_dir.path(), &bin_path, &options).unwrap();

    assert_eq!(discovered.parse_status, ParseStatus::Binary);
    assert!(discovered.file_hash.is_none());
}

#[test]
fn test_inventory_size_limit_prefilter() {
    let temp_dir = tempfile::tempdir().unwrap();
    let large_file = temp_dir.path().join("large.txt");

    // 200 bytes
    let data = vec![b'a'; 200];
    fs::write(&large_file, data).unwrap();

    // Limit set to 100 bytes
    let options = FileInventoryOptions {
        max_file_size_bytes: 100,
        ..Default::default()
    };

    let discovered = FileInventory::inspect_file(temp_dir.path(), &large_file, &options).unwrap();
    assert_eq!(discovered.parse_status, ParseStatus::TooLarge);
    assert!(discovered.file_hash.is_none());
}

#[test]
fn test_inventory_ignore_patterns() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("node_modules")).unwrap();
    fs::write(root.join("node_modules").join("dep.js"), "module.exports = {};\n").unwrap();

    fs::create_dir_all(root.join("target")).unwrap();
    fs::write(root.join("target").join("build.log"), "build done\n").unwrap();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("lib.rs"), "pub fn run() {}\n").unwrap();

    let summary = FileInventory::scan_dir(root, &[]).unwrap();
    assert_eq!(summary.total_files, 1);
    assert_eq!(summary.files[0].relative_path.to_str().unwrap(), "src/lib.rs".replace('/', std::path::MAIN_SEPARATOR_STR));
}

#[test]
fn test_inventory_invalid_utf8_failure_status() {
    let temp_dir = tempfile::tempdir().unwrap();
    let invalid_utf8_file = temp_dir.path().join("bad_encoding.txt");

    // Invalid UTF-8 sequence without null byte (0xFF, 0xFE, 0xFD)
    let bad_bytes = vec![0xFF, 0xFE, 0xFD, 0x80];
    fs::write(&invalid_utf8_file, bad_bytes).unwrap();

    let options = FileInventoryOptions::default();
    let discovered = FileInventory::inspect_file(temp_dir.path(), &invalid_utf8_file, &options).unwrap();

    assert_eq!(discovered.parse_status, ParseStatus::Failed);
    assert!(discovered.file_hash.is_some(), "File hash is still computed for failed encodings");
}
