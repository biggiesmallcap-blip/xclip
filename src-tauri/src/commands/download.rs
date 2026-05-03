use serde::Serialize;
use std::path::PathBuf;
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

enum YtDlpKind {
    Override(PathBuf),
    Backup(PathBuf),
    Bundled,
}

fn resolve_yt_dlp(app: &AppHandle) -> YtDlpKind {
    if let Ok(p) = paths::yt_dlp_override_path(app) {
        if p.exists() {
            eprintln!("[xclip:yt-dlp-resolve] using override at {}", p.display());
            return YtDlpKind::Override(p);
        }
    }
    if let Ok(p) = paths::yt_dlp_backup_path(app) {
        if p.exists() {
            eprintln!("[xclip:yt-dlp-resolve] override missing; falling back to backup at {}", p.display());
            return YtDlpKind::Backup(p);
        }
    }
    eprintln!("[xclip:yt-dlp-resolve] using bundled sidecar");
    YtDlpKind::Bundled
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

    let yt_dlp_args: Vec<String> = vec![
        "-f".into(),
        "bv*+ba/b".into(),
        "--merge-output-format".into(),
        "mp4".into(),
        "--no-playlist".into(),
        "--restrict-filenames".into(),
        "--no-warnings".into(),
        "--print".into(),
        "after_move:filepath".into(),
        "-o".into(),
        template_str,
        url.clone(),
    ];

    let (status_ok, stdout_str, stderr_str) = match resolve_yt_dlp(&app) {
        YtDlpKind::Override(p) | YtDlpKind::Backup(p) => {
            let out = tokio::process::Command::new(&p)
                .args(&yt_dlp_args)
                .output()
                .await
                .map_err(|e| format!("Failed to run yt-dlp: {e}"))?;
            (
                out.status.success(),
                String::from_utf8_lossy(&out.stdout).to_string(),
                String::from_utf8_lossy(&out.stderr).to_string(),
            )
        }
        YtDlpKind::Bundled => {
            let cmd = app
                .shell()
                .sidecar("yt-dlp")
                .map_err(|e| format!("yt-dlp sidecar not available: {e}"))?
                .args(&yt_dlp_args);
            let out = cmd
                .output()
                .await
                .map_err(|e| format!("Failed to run yt-dlp: {e}"))?;
            (
                out.status.success(),
                String::from_utf8_lossy(&out.stdout).to_string(),
                String::from_utf8_lossy(&out.stderr).to_string(),
            )
        }
    };

    if !status_ok {
        eprintln!("[xclip:yt-dlp-resolve] yt-dlp failed: {stderr_str}");
        return Err("Download failed. Check the URL or try again.".to_string());
    }

    let path = stdout_str
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
