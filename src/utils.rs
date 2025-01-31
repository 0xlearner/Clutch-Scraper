use crate::error::Result;
pub use crate::log_info;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn ensure_directory(dir: &str) -> Result<()> {
    if !Path::new(dir).exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub fn save_html(content: &str, page_number: usize) -> Result<PathBuf> {
    ensure_directory("local_html")?;

    let filename = format!("local_html/rust-page-{}.html", page_number);
    let path = PathBuf::from(&filename);

    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;

    log_info!("[utils] Saved HTML content to {}", filename);
    Ok(path)
}

pub fn read_html_files() -> Result<Vec<(PathBuf, String)>> {
    ensure_directory("local_html")?;

    let mut files = Vec::new();
    for entry in fs::read_dir("local_html")? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("html") {
            let content = fs::read_to_string(&path)?;
            files.push((path, content));
        }
    }

    // Sort files by page number
    files.sort_by(|(a_path, _), (b_path, _)| {
        let a_num = extract_page_number(a_path).unwrap_or(0);
        let b_num = extract_page_number(b_path).unwrap_or(0);
        a_num.cmp(&b_num)
    });

    Ok(files)
}

fn extract_page_number(path: &Path) -> Option<usize> {
    path.file_name().and_then(|n| n.to_str()).and_then(|name| {
        name.split('-')
            .nth(2)
            .and_then(|num| num.split('.').next())
            .and_then(|num| num.parse().ok())
    })
}

pub fn save_json(data: &impl serde::Serialize, path: impl AsRef<Path>) -> Result<()> {
    // Ensure the json_data directory exists
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let json_string = serde_json::to_string_pretty(data)?;
    let mut file = File::create(path)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}
