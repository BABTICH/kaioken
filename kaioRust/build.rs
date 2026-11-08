use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // WebView2Loader is statically linked on MSVC, but the GNU toolchain links
    // the import library. Place the matching loader beside the executable so
    // `cargo run` and direct launches behave identically. The WebView2 runtime
    // itself remains a Windows system component.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
        || env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("gnu")
    {
        return;
    }

    let folder = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "x64",
        Ok("x86") => "x86",
        Ok("aarch64") => "arm64",
        _ => return,
    };

    let Some(loader) = find_loader(folder) else {
        println!(
            "cargo:warning=WebView2Loader.dll was not found; place it beside the executable before launching the GUI."
        );
        return;
    };

    let Some(out_dir) = env::var_os("OUT_DIR").map(PathBuf::from) else {
        return;
    };
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .map(Path::to_path_buf)
        .unwrap_or(out_dir);

    for directory in [profile_dir.clone(), profile_dir.join("deps")] {
        let _ = fs::create_dir_all(&directory);
        let destination = directory.join("WebView2Loader.dll");
        let source_len = fs::metadata(&loader).map(|meta| meta.len()).unwrap_or(0);
        let destination_len = fs::metadata(&destination)
            .map(|meta| meta.len())
            .unwrap_or(0);
        if source_len == 0 || source_len != destination_len {
            let _ = fs::copy(&loader, &destination);
        }
    }
}

fn find_loader(folder: &str) -> Option<PathBuf> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR")?);
    let profile_dir = out_dir.ancestors().nth(3)?;

    // The dependency's build output when it has already run.
    if let Ok(entries) = fs::read_dir(profile_dir.join("build")) {
        for entry in entries.flatten() {
            let candidate = entry
                .path()
                .join("out")
                .join(folder)
                .join("WebView2Loader.dll");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // Fallback for clean builds where this build script runs before the
    // webview2-com-sys build script: read straight from the cargo registry.
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join(".cargo")))
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))?;
    let registry_src = cargo_home.join("registry").join("src");
    if let Ok(entries) = fs::read_dir(registry_src) {
        for entry in entries.flatten() {
            let candidate = entry
                .path()
                .join("webview2-com-sys-0.33.0")
                .join(folder)
                .join("WebView2Loader.dll");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}
