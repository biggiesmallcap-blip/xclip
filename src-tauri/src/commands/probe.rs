use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

#[derive(Serialize)]
pub struct ProbeResult {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

fn parse_rate(s: &str) -> f64 {
    if let Some((num, den)) = s.split_once('/') {
        let n: f64 = num.parse().unwrap_or(0.0);
        let d: f64 = den.parse().unwrap_or(1.0);
        if d > 0.0 { n / d } else { 0.0 }
    } else {
        s.parse().unwrap_or(0.0)
    }
}

fn parse_f64(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

#[tauri::command]
pub async fn probe(app: AppHandle, path: String) -> Result<ProbeResult, String> {
    if !std::path::Path::new(&path).exists() {
        return Err("Probe target file does not exist.".to_string());
    }

    let cmd = app
        .shell()
        .sidecar("ffprobe")
        .map_err(|e| format!("ffprobe sidecar not available: {e}"))?
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,r_frame_rate,duration",
            "-show_entries", "format=duration",
            "-of", "json",
            path.as_str(),
        ]);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to run ffprobe: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[xclip] ffprobe failed: {}", stderr);
        return Err("Could not probe the downloaded video.".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("ffprobe returned invalid JSON: {e}"))?;

    let stream = json
        .get("streams")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| "No video stream found.".to_string())?;

    let width = stream.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let height = stream.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let fps = stream
        .get("r_frame_rate")
        .and_then(|v| v.as_str())
        .map(parse_rate)
        .unwrap_or(0.0);

    let duration = stream
        .get("duration")
        .and_then(parse_f64)
        .or_else(|| json.get("format").and_then(|f| f.get("duration")).and_then(parse_f64))
        .unwrap_or(0.0);

    Ok(ProbeResult { duration, width, height, fps })
}
