mod commands;
mod focus;
mod paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::download::download,
            commands::probe::probe,
            commands::export::export_clip,
            commands::export::default_output_dir,
            commands::export::reveal_in_folder,
            commands::clipboard::copy_file_to_clipboard,
            commands::auto_track::auto_track,
        ])
        .run(tauri::generate_context!())
        .expect("error while running xclip");
}
