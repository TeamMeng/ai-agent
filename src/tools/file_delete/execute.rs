use std::{fs, path::Path};

pub fn delete(file_path: &str) -> anyhow::Result<String> {
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {file_path}");
    }
    if path.is_dir() {
        anyhow::bail!("Not a file: {file_path}");
    }

    fs::remove_file(path)?;
    Ok(format!("File {file_path} has been deleted."))
}
