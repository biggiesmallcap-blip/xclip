use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tauri::AppHandle;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::paths;

const CHECK_INTERVAL_DAYS: i64 = 7;
const GH_LATEST_URL: &str = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest";
const ASSET_NAME: &str = "yt-dlp.exe";
const MIN_BYTES: u64 = 5 * 1024 * 1024;
const MAX_BYTES: u64 = 100 * 1024 * 1024;
const USER_AGENT: &str = concat!("xclip/", env!("CARGO_PKG_VERSION"));

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct UpdateMeta {
    pub version: Option<String>,
    pub downloaded_at: Option<DateTime<Utc>>,
    pub last_checked_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UpdateStatus {
    Skipped { current_version: Option<String> },
    UpToDate { version: String },
    Updated { from: Option<String>, to: String },
    Failed { reason: String },
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

async fn read_meta(app: &AppHandle) -> UpdateMeta {
    let Ok(path) = paths::yt_dlp_meta_path(app) else { return UpdateMeta::default(); };
    let Ok(bytes) = fs::read(&path).await else { return UpdateMeta::default(); };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

async fn write_meta(app: &AppHandle, meta: &UpdateMeta) -> anyhow::Result<()> {
    let path = paths::yt_dlp_meta_path(app)?;
    let json = serde_json::to_vec_pretty(meta)?;
    fs::write(&path, json).await?;
    Ok(())
}

fn within_check_interval(meta: &UpdateMeta) -> bool {
    let Some(last) = meta.last_checked_at else { return false; };
    let elapsed = Utc::now().signed_duration_since(last);
    elapsed >= chrono::Duration::zero()
        && elapsed < chrono::Duration::days(CHECK_INTERVAL_DAYS)
}

async fn fetch_latest_release(client: &reqwest::Client) -> anyhow::Result<GhRelease> {
    let resp = client
        .get(GH_LATEST_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json::<GhRelease>().await?)
}

async fn stream_to_file(client: &reqwest::Client, url: &str, dest: &Path) -> anyhow::Result<u64> {
    let resp = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?;
    let mut file = fs::File::create(dest).await?;
    let mut stream = resp.bytes_stream();
    let mut total: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        total += bytes.len() as u64;
        file.write_all(&bytes).await?;
    }
    file.flush().await?;
    Ok(total)
}

/// Run the candidate exe with `--version` and confirm it returns a plausible version string.
pub async fn validate_yt_dlp_exe(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("missing file: {}", path.display()));
    }
    let meta = std::fs::metadata(path).map_err(|e| format!("metadata read: {e}"))?;
    if meta.len() < MIN_BYTES || meta.len() > MAX_BYTES {
        return Err(format!(
            "size out of range: {} bytes (expected {}–{})",
            meta.len(), MIN_BYTES, MAX_BYTES
        ));
    }
    let out = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new(path)
            .arg("--version")
            .output(),
    )
    .await
    .map_err(|_| "timed out running --version".to_string())?
    .map_err(|e| format!("spawn failed: {e}"))?;

    if !out.status.success() {
        return Err(format!("--version exit {}", out.status));
    }
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if stdout.is_empty() {
        return Err("empty --version output".to_string());
    }
    // yt-dlp prints lines like "2025.05.01" or "2025.05.01.123456" — accept anything starting with a digit.
    if !stdout.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return Err(format!("unexpected --version output: {stdout}"));
    }
    Ok(stdout)
}

async fn promote_with_rollback(
    app: &AppHandle,
    tmp: &Path,
) -> Result<(), String> {
    let dest = paths::yt_dlp_override_path(app).map_err(|e| e.to_string())?;
    let bak = paths::yt_dlp_backup_path(app).map_err(|e| e.to_string())?;
    if dest.exists() {
        let _ = std::fs::remove_file(&bak);
        std::fs::rename(&dest, &bak).map_err(|e| format!("backup rename: {e}"))?;
    }
    if let Err(e) = std::fs::rename(tmp, &dest) {
        eprintln!("[xclip:yt-dlp-update] promote failed: {e}; backup left at {}", bak.display());
        return Err(format!("promote rename: {e}"));
    }
    Ok(())
}

/// Top-level orchestrator. `force=true` ignores the 7-day window.
pub async fn run_check(app: &AppHandle, force: bool) -> UpdateStatus {
    let mut meta = read_meta(app).await;

    if !force && within_check_interval(&meta) {
        eprintln!("[xclip:yt-dlp-update] skipped (last_checked_at within {} days)", CHECK_INTERVAL_DAYS);
        return UpdateStatus::Skipped { current_version: meta.version.clone() };
    }

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[xclip:yt-dlp-update] http client build failed: {e}");
            return UpdateStatus::Failed { reason: format!("http client: {e}") };
        }
    };

    let release = match fetch_latest_release(&client).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[xclip:yt-dlp-update] github fetch failed: {e}");
            return UpdateStatus::Failed { reason: format!("github fetch: {e}") };
        }
    };

    let asset = match release.assets.iter().find(|a| a.name == ASSET_NAME) {
        Some(a) => a,
        None => {
            eprintln!("[xclip:yt-dlp-update] asset {ASSET_NAME} not found in latest release");
            return UpdateStatus::Failed { reason: format!("asset {ASSET_NAME} not found") };
        }
    };

    let latest_tag = release.tag_name.clone();

    if meta.version.as_deref() == Some(latest_tag.as_str()) {
        meta.last_checked_at = Some(Utc::now());
        if let Err(e) = write_meta(app, &meta).await {
            eprintln!("[xclip:yt-dlp-update] meta write (UpToDate) failed: {e}");
        }
        eprintln!("[xclip:yt-dlp-update] up-to-date at {latest_tag}");
        return UpdateStatus::UpToDate { version: latest_tag };
    }

    let dest = match paths::yt_dlp_override_path(app) {
        Ok(p) => p,
        Err(e) => return UpdateStatus::Failed { reason: format!("override path: {e}") },
    };
    let tmp = dest.with_extension("exe.tmp");
    let _ = std::fs::remove_file(&tmp);

    eprintln!("[xclip:yt-dlp-update] downloading {} ({})", asset.browser_download_url, latest_tag);
    let bytes_written = match stream_to_file(&client, &asset.browser_download_url, &tmp).await {
        Ok(n) => n,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            eprintln!("[xclip:yt-dlp-update] download failed: {e}");
            return UpdateStatus::Failed { reason: format!("download: {e}") };
        }
    };
    eprintln!("[xclip:yt-dlp-update] downloaded {bytes_written} bytes to {}", tmp.display());

    if let Err(reason) = validate_yt_dlp_exe(&tmp).await {
        let _ = std::fs::remove_file(&tmp);
        eprintln!("[xclip:yt-dlp-update] validation failed: {reason}");
        return UpdateStatus::Failed { reason: format!("validation: {reason}") };
    }

    if let Err(reason) = promote_with_rollback(app, &tmp).await {
        let _ = std::fs::remove_file(&tmp);
        return UpdateStatus::Failed { reason };
    }

    let now = Utc::now();
    let from = meta.version.clone();
    meta.version = Some(latest_tag.clone());
    meta.downloaded_at = Some(now);
    meta.last_checked_at = Some(now);
    if let Err(e) = write_meta(app, &meta).await {
        eprintln!("[xclip:yt-dlp-update] meta write (Updated) failed: {e}");
        // Update succeeded on disk; report Updated regardless.
    }

    eprintln!("[xclip:yt-dlp-update] updated {} -> {latest_tag}", from.clone().unwrap_or_else(|| "(none)".into()));
    UpdateStatus::Updated { from, to: latest_tag }
}

#[tauri::command]
pub async fn yt_dlp_check_update(app: AppHandle, force: Option<bool>) -> Result<UpdateStatus, String> {
    Ok(run_check(&app, force.unwrap_or(false)).await)
}

#[tauri::command]
pub async fn yt_dlp_update_now(app: AppHandle) -> Result<UpdateStatus, String> {
    Ok(run_check(&app, true).await)
}

/// Verify the override is healthy. If broken, move it aside so resolution falls through.
/// Cheap startup task — runs after the background update check.
pub async fn self_test_override(app: &AppHandle) {
    let Ok(path) = paths::yt_dlp_override_path(app) else { return; };
    if !path.exists() { return; }
    if let Err(reason) = validate_yt_dlp_exe(&path).await {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let broken = path.with_file_name(format!("yt-dlp.exe.broken-{stamp}"));
        eprintln!("[xclip:yt-dlp-resolve] override unhealthy ({reason}); moving aside to {}", broken.display());
        let _ = std::fs::rename(&path, &broken);
    }
}
