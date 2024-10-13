pub fn find_rs_files(path: &Path) -> Vec<PathBuf> {
    let mut rs_files = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() && !entry_path.to_string_lossy().contains("/context") {
                    rs_files.extend(find_rs_files(&entry_path));
                } else if let Some(extension) = entry_path.extension() {
                    if extension == "rs" {
                        rs_files.push(entry_path);
                    }
                }
            }
        }
    }
    rs_files
}
