use std::path::Path;

pub fn hello_world_python() -> &'static str {
    "def main() -> None:\n    print(\"hello from Call\")\n\n\nif __name__ == \"__main__\":\n    main()\n"
}

pub fn is_empty_of_source(root: &Path) -> bool {
    !contains_py(root)
}

fn contains_py(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if name.starts_with('.') || name == "venv" || name == "__pycache__" {
                continue;
            }
            if contains_py(&path) {
                return true;
            }
        } else if name.ends_with(".py") {
            return true;
        }
    }
    false
}

pub fn scaffold(root: &Path, language: &str) -> Result<String, String> {
    if language != "python" {
        return Err(format!("unsupported scaffold language: {language}"));
    }
    if !is_empty_of_source(root) {
        return Err("workspace already has source files; pick an Entry instead of scaffolding".into());
    }
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let path = root.join("main.py");
    std::fs::write(&path, hello_world_python()).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
