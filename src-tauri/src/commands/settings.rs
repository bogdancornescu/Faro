use tauri_plugin_global_shortcut::GlobalShortcutExt;
use crate::error::AppError;

#[tauri::command]
pub fn update_hotkey(app: tauri::AppHandle, hotkey: String) -> Result<(), AppError> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| AppError::Other(e.to_string()))?;
    app.global_shortcut()
        .register(hotkey.as_str())
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}
