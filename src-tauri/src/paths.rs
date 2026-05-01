use anyhow::Result;
use std::path::PathBuf;

// Project-local folders so everything stays on the workdrive next to the source.
const DOWNLOADS_DIR: &str = r"E:\xclip\downloads";
const OUTPUT_DIR: &str = r"E:\xclip\output";

pub fn cache_dir() -> Result<PathBuf> {
    let dir = PathBuf::from(DOWNLOADS_DIR);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn output_dir() -> Result<PathBuf> {
    let dir = PathBuf::from(OUTPUT_DIR);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn timestamped(prefix: &str, ext: &str) -> String {
    format!(
        "{prefix}-{}.{ext}",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    )
}
