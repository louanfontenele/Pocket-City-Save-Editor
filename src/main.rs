#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod catalog;
mod editor;
mod locale;
mod progression_catalog;
mod save;

use gpui_kit::{
    AppContext, Bounds, TitlebarOptions, WindowBounds, WindowOptions, application,
    component::{Theme, ThemeMode},
    init, open_window, px, rgb, size,
};

fn main() {
    application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            init(cx);
            Theme::change(ThemeMode::Dark, None, cx);
            Theme::update(cx, |theme| {
                theme.colors.background = rgb(0x181c22).into();
                theme.colors.sidebar = rgb(0x12161b).into();
                theme.colors.sidebar_accent = rgb(0x263642).into();
                theme.colors.sidebar_border = rgb(0x323c47).into();
                theme.colors.border = rgb(0x323c47).into();
                theme.colors.input = rgb(0x303945).into();
                theme.colors.muted = rgb(0x232b34).into();
                theme.colors.tab_bar = rgb(0x232b34).into();
                theme.colors.tab_active_foreground = rgb(0xa7ddf2).into();
            });
            let bounds = Bounds::centered(None, size(px(1120.0), px(740.0)), cx);
            open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Pocket City Save Editor".into()),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(780.0), px(540.0))),
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| editor::Editor::new(window, cx)),
            )
            .expect("Failed to open Pocket City Save Editor");
            cx.activate(true);
        });
}
