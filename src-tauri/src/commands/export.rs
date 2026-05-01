use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::paths;
use crate::focus::{FocusTools, build_filter_plan, FilterPlan};

#[tauri::command]
pub fn default_output_dir() -> Result<String, String> {
    paths::output_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reveal_in_folder(path: String) -> Result<(), String> {
    let p = std::path::PathBuf::from(&path);
    if !p.exists() {
        return Err("Folder does not exist.".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&p)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {e}"))?;
        return Ok(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Reveal not supported on this platform".to_string())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportArgs {
    pub input_path: String,
    pub start: f64,
    pub end: f64,
    pub format: String,
    pub quality: String,
    #[serde(default)]
    pub focus_tools: Option<FocusTools>,
    #[serde(default)]
    pub output_dir: Option<String>,
}

#[derive(Serialize)]
pub struct ExportResult {
    #[serde(rename = "outputPath")]
    pub output_path: String,
}

struct Mp4Preset { width: Option<u32>, crf: u32 }
struct GifPreset { width: u32, fps: u32 }

fn mp4_preset(quality: &str) -> Result<Mp4Preset, String> {
    match quality {
        "small"    => Ok(Mp4Preset { width: Some(480), crf: 28 }),
        "balanced" => Ok(Mp4Preset { width: Some(720), crf: 23 }),
        "hq"       => Ok(Mp4Preset { width: None,      crf: 20 }),
        _ => Err("Invalid quality. Use small, balanced, or hq.".to_string()),
    }
}

fn gif_preset(quality: &str) -> Result<GifPreset, String> {
    match quality {
        "small"    => Ok(GifPreset { width: 360, fps: 12 }),
        "balanced" => Ok(GifPreset { width: 480, fps: 18 }),
        "hq"       => Ok(GifPreset { width: 640, fps: 24 }),
        _ => Err("Invalid quality. Use small, balanced, or hq.".to_string()),
    }
}

async fn run_ffmpeg(app: &AppHandle, args: Vec<String>) -> Result<(), String> {
    let cmd = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("ffmpeg sidecar not available: {e}"))?
        .args(args);

    let (mut rx, _child) = cmd
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {e}"))?;

    let mut stderr_buf = String::new();
    let mut exit_code: Option<i32> = None;

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stderr(line) => {
                stderr_buf.push_str(&String::from_utf8_lossy(&line));
                stderr_buf.push('\n');
            }
            CommandEvent::Stdout(_) => {}
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
            }
            CommandEvent::Error(e) => {
                return Err(format!("ffmpeg error: {e}"));
            }
            _ => {}
        }
    }

    match exit_code {
        Some(0) => Ok(()),
        Some(code) => {
            eprintln!("[xclip] ffmpeg exit {code}: {stderr_buf}");
            let tail: String = stderr_buf
                .lines()
                .rev()
                .filter(|l| !l.trim().is_empty())
                .take(6)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            Err(format!("ffmpeg failed (exit {code}):\n{tail}"))
        }
        None => Err("ffmpeg terminated unexpectedly.".to_string()),
    }
}

async fn probe_size(app: &AppHandle, path: &str) -> Result<(u32, u32), String> {
    let cmd = app
        .shell()
        .sidecar("ffprobe")
        .map_err(|e| format!("ffprobe sidecar not available: {e}"))?
        .args(vec![
            "-v".to_string(), "error".to_string(),
            "-select_streams".to_string(), "v:0".to_string(),
            "-show_entries".to_string(), "stream=width,height".to_string(),
            "-of".to_string(), "csv=p=0:s=x".to_string(),
            path.to_string(),
        ]);
    let (mut rx, _child) = cmd.spawn().map_err(|e| format!("Failed to start ffprobe: {e}"))?;
    let mut out = String::new();
    while let Some(ev) = rx.recv().await {
        if let CommandEvent::Stdout(line) = ev {
            out.push_str(&String::from_utf8_lossy(&line));
        }
    }
    let trimmed = out.trim();
    let parts: Vec<&str> = trimmed.split('x').collect();
    if parts.len() != 2 {
        return Err(format!("Could not parse video size from ffprobe: '{trimmed}'"));
    }
    let w: u32 = parts[0].trim().parse().map_err(|_| "Bad width from ffprobe".to_string())?;
    let h: u32 = parts[1].trim().parse().map_err(|_| "Bad height from ffprobe".to_string())?;
    Ok((w, h))
}

fn focus_active(focus: &Option<FocusTools>) -> bool {
    focus.as_ref().map(|f| f.enabled && f.mode != "off").unwrap_or(false)
}

/// Write a filter chain to a temp script file when it's too long for the command line.
/// Returns the path on disk; caller is responsible for cleanup.
fn write_filter_script(content: &str, label: &str) -> Result<std::path::PathBuf, String> {
    let cache = paths::cache_dir().map_err(|e| e.to_string())?;
    let path = cache.join(paths::timestamped(&format!("filter-{label}"), "txt"));
    std::fs::write(&path, content).map_err(|e| format!("Failed to write filter script: {e}"))?;
    Ok(path)
}

/// Render an intermediate MP4 of the trim window with the focus filter applied.
/// The output's keyframes are anchored at t=0 (i.e. start_time has been baked out),
/// so the caller can run the rest of the export pipeline at start=0, end=(end-start)
/// with no further focus processing.
async fn render_focus_intermediate(
    app: &AppHandle,
    args: &ExportArgs,
    source: (u32, u32),
) -> Result<std::path::PathBuf, String> {
    let focus = args.focus_tools.as_ref().unwrap();
    let cache = paths::cache_dir().map_err(|e| e.to_string())?;
    let intermediate = cache.join(paths::timestamped("focus-intermediate", "mp4"));
    let intermediate_str = intermediate.to_string_lossy().to_string();

    let plan = build_filter_plan(focus, source, args.start, None)?;
    let graph = format!("[0:v]{}[v]", plan.filter);
    let script = write_filter_script(&graph, "focus")?;
    let script_str = script.to_string_lossy().to_string();

    let ff_args: Vec<String> = vec![
        "-y".into(),
        "-ss".into(), format!("{:.3}", args.start),
        "-to".into(), format!("{:.3}", args.end),
        "-i".into(), args.input_path.clone(),
        "-filter_complex_script".into(), script_str,
        "-map".into(), "[v]".into(),
        "-map".into(), "0:a?".into(),
        "-c:v".into(), "libx264".into(),
        "-preset".into(), "ultrafast".into(),
        "-crf".into(), "18".into(),
        "-pix_fmt".into(), "yuv420p".into(),
        "-c:a".into(), "aac".into(),
        "-b:a".into(), "160k".into(),
        intermediate_str.clone(),
    ];

    let result = run_ffmpeg(app, ff_args.clone()).await;
    match result {
        Ok(()) => {
            let _ = std::fs::remove_file(&script);
            Ok(intermediate)
        }
        Err(e) => {
            let _ = std::fs::remove_file(&intermediate);
            // Preserve the script for inspection on failure and surface its path.
            let kept = cache.join("last-failed-filter.txt");
            let _ = std::fs::copy(&script, &kept);
            let _ = std::fs::remove_file(&script);
            // Also write the full ffmpeg arg list for diagnosis.
            let cmd_path = cache.join("last-failed-cmd.txt");
            let cmd_str = ff_args.iter()
                .map(|a| if a.contains(' ') { format!("\"{}\"", a) } else { a.clone() })
                .collect::<Vec<_>>()
                .join(" ");
            let _ = std::fs::write(&cmd_path, &cmd_str);
            Err(format!(
                "{}\n\nFilter saved: {}\nCommand saved: {}\nKeyframes: {}",
                e,
                kept.display(),
                cmd_path.display(),
                focus.keyframes.len(),
            ))
        }
    }
}

#[tauri::command]
pub async fn export_clip(app: AppHandle, args: ExportArgs) -> Result<ExportResult, String> {
    if !std::path::Path::new(&args.input_path).exists() {
        return Err("Input file no longer exists.".to_string());
    }
    if args.end <= args.start {
        return Err("End must be greater than start.".to_string());
    }
    if args.start < 0.0 {
        return Err("Start must be non-negative.".to_string());
    }

    let out_dir = match args.output_dir.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(custom) => {
            let pb = std::path::PathBuf::from(custom);
            std::fs::create_dir_all(&pb).map_err(|e| format!("Couldn't create output folder: {e}"))?;
            pb
        }
        None => paths::output_dir().map_err(|e| e.to_string())?,
    };

    let focus_on = focus_active(&args.focus_tools);

    // When focus is active, render a focused intermediate first, then run the simple
    // (proven) export pipeline against that intermediate. Keeps the brittle filter
    // graph isolated to one stage.
    let (input_path, eff_start, eff_end, intermediate): (String, f64, f64, Option<std::path::PathBuf>) = if focus_on {
        let src = probe_size(&app, &args.input_path).await?;
        let inter = render_focus_intermediate(&app, &args, src).await?;
        let path = inter.to_string_lossy().to_string();
        // The intermediate is already trimmed to [start, end], so subsequent pipeline
        // operates on [0, end-start].
        (path, 0.0, args.end - args.start, Some(inter))
    } else {
        (args.input_path.clone(), args.start, args.end, None)
    };

    let start_str = format!("{:.3}", eff_start);
    let end_str = format!("{:.3}", eff_end);

    let cleanup = |p: &Option<std::path::PathBuf>| {
        if let Some(p) = p { let _ = std::fs::remove_file(p); }
    };

    let result = match args.format.as_str() {
        "mp4" => {
            let p = mp4_preset(&args.quality)?;
            let out_path = out_dir.join(paths::timestamped("clip", "mp4"));
            let out_str = out_path.to_string_lossy().to_string();

            let mut a: Vec<String> = vec![
                "-y".into(),
                "-ss".into(), start_str.clone(),
                "-to".into(), end_str.clone(),
                "-i".into(), input_path.clone(),
            ];
            if let Some(w) = p.width {
                a.push("-vf".into());
                a.push(format!("scale={}:-2", w));
            }
            a.extend([
                "-c:v".into(), "libx264".into(),
                "-crf".into(), p.crf.to_string(),
                "-preset".into(), "veryfast".into(),
                "-pix_fmt".into(), "yuv420p".into(),
                "-movflags".into(), "+faststart".into(),
                "-c:a".into(), "aac".into(),
                "-b:a".into(), "128k".into(),
                out_str.clone(),
            ]);
            run_ffmpeg(&app, a).await.map(|_| ExportResult { output_path: out_str })
        }
        "gif" => {
            let p = gif_preset(&args.quality)?;
            let cache = paths::cache_dir().map_err(|e| e.to_string())?;
            let palette_path = cache.join(paths::timestamped("palette", "png"));
            let palette_str = palette_path.to_string_lossy().to_string();

            let out_path = out_dir.join(paths::timestamped("clip", "gif"));
            let out_str = out_path.to_string_lossy().to_string();

            let filter_pal = format!(
                "fps={},scale={}:-1:flags=lanczos,palettegen",
                p.fps, p.width
            );
            let pass1: Vec<String> = vec![
                "-y".into(),
                "-ss".into(), start_str.clone(),
                "-to".into(), end_str.clone(),
                "-i".into(), input_path.clone(),
                "-vf".into(), filter_pal,
                "-update".into(), "1".into(),
                "-frames:v".into(), "1".into(),
                palette_str.clone(),
            ];
            if let Err(e) = run_ffmpeg(&app, pass1).await {
                let _ = std::fs::remove_file(&palette_path);
                return Err(e).map_err(|e| { cleanup(&intermediate); e });
            }

            let filter_use = format!(
                "fps={},scale={}:-1:flags=lanczos [x]; [x][1:v] paletteuse",
                p.fps, p.width
            );
            let pass2: Vec<String> = vec![
                "-y".into(),
                "-ss".into(), start_str,
                "-to".into(), end_str,
                "-i".into(), input_path.clone(),
                "-i".into(), palette_str.clone(),
                "-lavfi".into(), filter_use,
                out_str.clone(),
            ];
            let pass2_result = run_ffmpeg(&app, pass2).await;
            let _ = std::fs::remove_file(&palette_path);
            pass2_result.map(|_| ExportResult { output_path: out_str })
        }
        other => Err(format!("Invalid format '{other}'. Use mp4 or gif.")),
    };

    cleanup(&intermediate);
    result
}
