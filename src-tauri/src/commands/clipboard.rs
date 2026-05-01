#[cfg(windows)]
#[tauri::command]
pub fn copy_file_to_clipboard(path: String) -> Result<(), String> {
    use clipboard_win::{formats, Clipboard, Setter};

    if !std::path::Path::new(&path).exists() {
        return Err("File does not exist.".to_string());
    }

    let _clip = Clipboard::new_attempts(10)
        .map_err(|e| format!("Clipboard busy: {e}"))?;

    formats::FileList
        .write_clipboard(&[path.as_str()])
        .map_err(|e| format!("Clipboard write failed: {e}"))?;

    Ok(())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn copy_file_to_clipboard(_path: String) -> Result<(), String> {
    Err("File-to-clipboard copy is only implemented on Windows.".to_string())
}
