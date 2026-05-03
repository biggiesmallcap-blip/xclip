mod commands;
mod focus;
mod paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Background tasks: yt-dlp self-test, then once-per-week update check.
            // Both are non-fatal — failures stay on stderr and the bundled sidecar
            // remains the fallback.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                commands::yt_dlp_update::self_test_override(&handle).await;
                let _ = commands::yt_dlp_update::run_check(&handle, false).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::download::download,
            commands::probe::probe,
            commands::export::export_clip,
            commands::export::default_output_dir,
            commands::export::reveal_in_folder,
            commands::clipboard::copy_file_to_clipboard,
            commands::auto_track::auto_track,
            commands::yt_dlp_update::yt_dlp_check_update,
            commands::yt_dlp_update::yt_dlp_update_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running xclip");
}
