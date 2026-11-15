use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn unzip(zip_path: &str, extract_to: Option<&str>) -> anyhow::Result<String> {
    let zip_path = Path::new(zip_path);
    if !zip_path.exists() {
        anyhow::bail!("File not found: {}", zip_path.display());
    }

    let extract_to: PathBuf = match extract_to {
        Some(dir) => PathBuf::from(dir),
        None => zip_path.with_extension(""),
    };
    fs::create_dir_all(&extract_to)?;

    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut names = Vec::with_capacity(archive.len());
    let mut file_count = 0usize;
    let mut skipped = 0usize;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        // `enclosed_name` rejects absolute paths and `..` traversal, so a
        // malicious archive cannot write outside `extract_to`.
        let Some(relative) = entry.enclosed_name() else {
            skipped += 1;
            continue;
        };
        let out_path = extract_to.join(&relative);
        names.push(relative.display().to_string());

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut out_file)?;
            file_count += 1;
        }
    }

    let mut summary = format!(
        "Extracted {} file(s) to {}/\n\nContents:\n",
        file_count,
        extract_to.display()
    );

    for name in names.iter().take(20) {
        summary.push_str(&format!(". - {name}\n"));
    }

    if names.len() > 20 {
        summary.push_str(&format!(" ... and {} more files\n", names.len() - 20));
    }

    if skipped > 0 {
        summary.push_str(&format!(
            "\nSkipped {skipped} unsafe entry(ies) that would escape the target directory.\n"
        ));
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ai-agent-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    #[test]
    fn skips_entries_that_escape_the_target_directory() -> anyhow::Result<()> {
        let root = scratch_dir("unzip-traversal");
        let zip_path = root.join("archive.zip");

        let mut writer = zip::ZipWriter::new(fs::File::create(&zip_path)?);
        let options = zip::write::SimpleFileOptions::default();
        writer.start_file("safe.txt", options)?;
        writer.write_all(b"hello")?;
        writer.start_file("../escaped.txt", options)?;
        writer.write_all(b"pwned")?;
        writer.finish()?;

        let extract_to = root.join("out");
        let summary = unzip(
            zip_path.to_str().expect("zip path is utf-8"),
            Some(extract_to.to_str().expect("extract path is utf-8")),
        )?;

        assert!(extract_to.join("safe.txt").exists());
        assert!(!root.join("escaped.txt").exists());
        assert!(summary.contains("Skipped 1 unsafe entry"));

        fs::remove_dir_all(&root)?;
        Ok(())
    }
}
