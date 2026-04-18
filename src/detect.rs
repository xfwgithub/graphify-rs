use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub fn is_valid_file_type(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext, "py" | "rs" | "go" | "js" | "ts" | "md" | "java" | "rb" | "cpp" | "c" | "h" | "cs" | "kt" | "scala" | "php")
    } else {
        false
    }
}

pub fn scan_directory(target_dir: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let target_path = Path::new(target_dir);

    let mut builder = WalkBuilder::new(target_path);
    builder.hidden(true)
           .git_ignore(true)
           .filter_entry(|entry| {
               let fname = entry.file_name().to_string_lossy();
               !fname.starts_with('.') && 
               fname != "node_modules" && 
               fname != "target" &&
               fname != "build" &&
               fname != "dist" &&
               fname != "out" &&
               fname != "venv"
           });

    for result in builder.build() {
        match result {
            Ok(entry) => {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    let path = entry.path();
                    if is_valid_file_type(path) {
                        files.push(path.to_path_buf());
                    }
                }
            }
            Err(_) => continue,
        }
    }
    
    Ok(files)
}
