#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod catalog;
mod editor;
mod locale;
mod progression_catalog;
mod save;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        centered: true,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 680.0])
            .with_min_inner_size([780.0, 540.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Pocket City Save Editor",
        options,
        Box::new(|creation| Ok(Box::new(editor::Editor::new(creation)))),
    )
}
