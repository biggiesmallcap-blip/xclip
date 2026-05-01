use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use url::Url;

use crate::paths;

const ALLOWED_HOSTS: &[&str] = &[
    "x.com",
    "twitter.com",
    "youtube.com",
    "youtu.be",
    "reddit.com",
    "tiktok.com",
    "vm.tiktok.com",
];

#[derive(Serialize)]
pub struct DownloadResult {
    pub path: String,
}

fn host_is_allowed(host: &str) -> bool {
    let h = host.trim_start_matches("www.").to_ascii_lowercase();
    ALLOWED_HOSTS.iter().any(|allowed| {
        h == *allowed || h.ends_with(&format!(".{allowed}"))
    })
}

#[tauri::command]
pub async fn download(app: AppHandle, url: String) -> Result<DownloadResult, String> {
    let parsed = Url::parse(&url).map_err(|_| "Invalid URL.".to_string())?;
    let host = parsed
        .host_str()
        .ok_or_else(|| "URL is missing a host.".to_string())?
        .to_string();

    if !host_is_allowed(&host) {
        return Err(format!(
            "Unsupported site: {host}. Supported: x.com, twitter.com, youtube.com, youtu.be, reddit.com, tiktok.com, vm.tiktok.com"
        ));
    }

    let cache = paths::cache_dir().map_err(|e| e.to_string())?;
    let template = cache.join("%(id)s-%(epoch)s.%(ext)s");
    let template_str = template.to_string_lossy().to_string();

    let cmd = app
        .shell()
        .sidecar("yt-dlp")
        .map_err(|e| format!("yt-dlp sidecar not available: {e}"))?
        .args([
            "-f",
            "bv*+ba/b",
            "--merge-output-format",
            "mp4",
            "--no-playlist",
            "--restrict-filenames",
            "--no-warnings",
            "--print",
            "after_move:filepath",
            "-o",
            template_str.as_str(),
            url.as_str(),
        ]);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[xclip] yt-dlp failed: {}", stderr);
        return Err("Download failed. Check the URL or try again.".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .last()
        .ok_or_else(|| "yt-dlp finished but did not report an output path.".to_string())?
        .to_string();

    if !std::path::Path::new(&path).exists() {
        return Err(format!("Downloaded file not found at {path}"));
    }

    Ok(DownloadResult { path })
}
