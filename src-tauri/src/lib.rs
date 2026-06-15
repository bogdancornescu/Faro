use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub mod commands;
pub mod db;
pub mod error;
pub mod models;

fn open_quick_note(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("quicknote") {
        let _ = w.set_focus();
        return;
    }
    match tauri::WebviewWindowBuilder::new(
        app,
        "quicknote",
        tauri::WebviewUrl::App("quicknote".into()),
    )
    .title("Quick Note")
    .inner_size(600.0, 480.0)
    .always_on_top(true)
    .center()
    .resizable(false)
    .build()
    {
        Ok(w) => { let _ = w.set_focus(); }
        Err(e) => eprintln!("[faro] failed to open quick-note window: {e}"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        open_quick_note(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("faro.db");
            let conn = db::open_db(&db_path)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            app.manage(Mutex::new(conn));
            app.global_shortcut().register("Ctrl+Shift+Space")?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::snippets::create_snippet,
            commands::snippets::list_snippets,
            commands::snippets::get_snippet,
            commands::snippets::update_snippet,
            commands::snippets::delete_snippet,
            commands::snippets::search_snippets,
            commands::snippets::list_snippets_by_period,
            commands::snippets::record_copy,
            commands::tags::list_tags,
            commands::settings::update_hotkey,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
