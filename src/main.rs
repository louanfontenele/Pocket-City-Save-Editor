#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod catalog;
mod editor;
mod locale;
mod progression_catalog;
mod save;

use std::sync::Mutex;

use editor::{Editor, EditorState};
use tauri::{DragDropEvent, Emitter, Manager, WindowEvent};

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(Editor::new()))
        .invoke_handler(tauri::generate_handler![editor::editor_request])
        .on_window_event(|window, event| {
            if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event
                && let Some(path) = paths.first().cloned()
            {
                let app = window.app_handle().clone();
                tauri::async_runtime::spawn_blocking(move || {
                    if let Ok(mut editor) = app.state::<EditorState>().lock() {
                        editor.queue_drop(path);
                    }
                    let _ = app.emit("editor-updated", ());
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("Failed to open Pocket City Save Editor");
}
