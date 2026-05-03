use anyhow::Result;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

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

/// Per-user app data directory (Windows: %APPDATA%\dev.xclip.app\).
pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("app_data_dir unavailable: {e}"))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Auto-updated yt-dlp.exe location. Writable, persistent across reinstalls.
pub fn yt_dlp_override_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app_data_dir(app)?.join("bin");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("yt-dlp.exe"))
}

/// Sibling rollback copy of the previous override.
pub fn yt_dlp_backup_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(app_data_dir(app)?.join("bin").join("yt-dlp.exe.bak"))
}

/// Update metadata (version + timestamps) sidecar to the override.
pub fn yt_dlp_meta_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(app_data_dir(app)?.join("yt-dlp-update.json"))
}
