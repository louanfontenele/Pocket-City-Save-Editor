use crate::{
    catalog::{Collection, MAX_AFFINITY},
    locale::Locale,
    save::{self, Game, Kind, Save},
};
use eframe::egui::{self, Align, Color32, Frame, Layout, Margin, RichText, Stroke, TextEdit};
use std::path::{Path, PathBuf};

const BG: Color32 = Color32::from_rgb(19, 25, 39);
const SIDEBAR: Color32 = Color32::from_rgb(24, 32, 49);
const SURFACE: Color32 = Color32::from_rgb(31, 42, 63);
const HEADER: Color32 = Color32::from_rgb(36, 50, 77);
const BORDER: Color32 = Color32::from_rgb(57, 74, 105);
const MUTED: Color32 = Color32::from_rgb(172, 188, 212);
const INK: Color32 = Color32::from_rgb(249, 246, 232);
const WARM: Color32 = Color32::from_rgb(255, 198, 93);
const ERROR: Color32 = Color32::from_rgb(255, 166, 154);
const POCKET_CITY_COLORS: [Color32; 3] = [
    Color32::from_rgb(147, 167, 241),
    WARM,
    Color32::from_rgb(163, 205, 99),
];
const POCKET_CITY_2_COLORS: [Color32; 3] = [
    Color32::from_rgb(100, 206, 242),
    Color32::from_rgb(255, 159, 126),
    Color32::from_rgb(168, 221, 118),
];

fn logo_colors(game: Game) -> [Color32; 3] {
    match game {
        Game::One => POCKET_CITY_COLORS,
        Game::Two => POCKET_CITY_2_COLORS,
    }
}

struct Entry {
    path: PathBuf,
    title: String,
    game: Game,
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    City,
    Collections,
    Advanced,
    Backups,
}

pub struct Editor {
    game: Game,
    library: Vec<Entry>,
    folders: Vec<PathBuf>,
    current: Option<Save>,
    query: String,
    tab: Tab,
    notice: Option<(bool, String)>,
    restore: Option<PathBuf>,
    snapshot: usize,
    collection: Collection,
    locale: Locale,
}

impl Editor {
    pub fn new(creation: &eframe::CreationContext<'_>) -> Self {
        let locale = creation
            .storage
            .and_then(|storage| storage.get_string("locale"))
            .map_or_else(Locale::default, |code| Locale::from_code(&code));
        locale.install_fonts(&creation.egui_ctx);
        let mut style = (*creation.egui_ctx.style()).clone();
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(INK);
        visuals.panel_fill = BG;
        visuals.window_fill = SURFACE;
        visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
        visuals.extreme_bg_color = BG;
        visuals.selection.bg_fill = Color32::from_rgb(42, 70, 105);
        visuals.selection.stroke = Stroke::new(1.0_f32, POCKET_CITY_2_COLORS[0]);
        visuals.faint_bg_color = Color32::from_rgb(37, 49, 71);
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.weak_bg_fill = SURFACE;
            widget.bg_fill = SURFACE;
            widget.bg_stroke = Stroke::new(1.0_f32, BORDER);
            widget.corner_radius = 6.into();
        }
        visuals.widgets.hovered.bg_fill = BORDER;
        visuals.widgets.hovered.weak_bg_fill = BORDER;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, POCKET_CITY_2_COLORS[0]);
        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.interact_size.y = 31.0;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
        creation.egui_ctx.set_style(style);

        let mut editor = Self {
            game: Game::Two,
            library: Vec::new(),
            folders: save::default_folders()
                .into_iter()
                .filter(|p| p.is_dir())
                .collect(),
            current: None,
            query: String::new(),
            tab: Tab::City,
            notice: None,
            restore: None,
            snapshot: 0,
            collection: Collection::Affinity,
            locale,
        };
        editor.refresh();
        editor.game = editor.library.first().map_or(Game::One, |entry| entry.game);
        if let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) {
            editor.open(&path);
        } else if let Some(path) = editor
            .library
            .iter()
            .filter(|entry| archive_key(&entry.path) == "main")
            .max_by_key(|entry| {
                entry
                    .path
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .ok()
            })
            .map(|entry| entry.path.clone())
        {
            editor.open(&path);
        }
        editor
    }

    fn refresh(&mut self) {
        for folder in self.folders.clone() {
            match save::scan(&folder) {
                Ok(paths) => {
                    for path in paths {
                        if let Ok(document) = Save::open(&path) {
                            self.remember(&document);
                        }
                    }
                }
                Err(error) => {
                    self.status(true, format!("{}\n{error:#}", self.locale.text("failed")))
                }
            }
        }
        self.library.retain(|entry| entry.path.is_file());
        self.library.sort_by(|a, b| {
            a.title
                .to_lowercase()
                .cmp(&b.title.to_lowercase())
                .then_with(|| a.path.cmp(&b.path))
        });
    }

    fn remember(&mut self, document: &Save) {
        if let Some(entry) = self
            .library
            .iter_mut()
            .find(|entry| entry.path == document.path)
        {
            entry.title.clone_from(&document.title);
        } else {
            self.library.push(Entry {
                path: document.path.clone(),
                title: document.title.clone(),
                game: document.game,
            });
        }
    }

    fn status(&mut self, error: bool, message: impl Into<String>) {
        self.notice = Some((error, message.into()));
    }

    fn open(&mut self, path: &Path) {
        let locale = self.locale;
        if self
            .current
            .as_ref()
            .is_some_and(|save| save.changed() != 0)
        {
            self.status(true, locale.text("pending_guard"));
            return;
        }
        match Save::open(path) {
            Ok(document) => {
                self.game = document.game;
                self.snapshot = document.latest;
                self.remember(&document);
                self.tab = if document
                    .fields
                    .iter()
                    .any(|field| field_section(&field.path) == Tab::City)
                {
                    Tab::City
                } else {
                    Tab::Advanced
                };
                self.current = Some(document);
                self.query.clear();
                self.notice = None;
            }
            Err(error) => self.status(true, format!("{}\n{error:#}", self.locale.text("failed"))),
        }
    }

    fn sidebar(&mut self, ctx: &egui::Context) {
        let locale = self.locale;
        egui::SidePanel::left("library")
            .exact_width(262.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(SIDEBAR)
                    .inner_margin(Margin::same(18))
                    .stroke(Stroke::new(1.0_f32, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(33.0, 33.0), egui::Sense::hover());
                    let paint = ui.painter();
                    paint.rect_filled(rect, 8.0, HEADER);
                    let colors = logo_colors(self.game);
                    for ((x, height), color) in [(7.0, 10.0), (14.0, 20.0), (21.0, 15.0)]
                        .into_iter()
                        .zip(colors)
                    {
                        let origin = rect.left_bottom() + egui::vec2(x, -7.0 - height);
                        paint.rect_filled(
                            egui::Rect::from_min_size(origin, egui::vec2(5.0, height)),
                            1.0,
                            color,
                        );
                    }
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Pocket City").strong().size(20.0));
                        ui.label(RichText::new(locale.text("app")).color(MUTED).size(13.0));
                    });
                });
                ui.horizontal(|ui| {
                    ui.label(locale.text("language"));
                    egui::ComboBox::from_id_salt("editor-language")
                        .selected_text(self.locale.name())
                        .show_ui(ui, |ui| {
                            for (code, name) in Locale::ALL {
                                ui.selectable_value(
                                    &mut self.locale,
                                    Locale::from_code(code),
                                    name,
                                );
                            }
                        });
                });
                if self.locale != locale {
                    self.locale.install_fonts(ctx);
                    self.query.clear();
                    self.notice = None;
                    ctx.request_repaint();
                }
                ui.add_space(22.0);
                for game in [Game::One, Game::Two] {
                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            egui::Button::selectable(self.game == game, game.title()),
                        )
                        .clicked()
                    {
                        if self
                            .current
                            .as_ref()
                            .is_some_and(|save| save.game != game && save.changed() != 0)
                        {
                            self.status(true, locale.text("pending_guard"));
                        } else {
                            self.game = game;
                            if self.current.as_ref().is_some_and(|save| save.game != game) {
                                self.current = None;
                            }
                        }
                    }
                }
                ui.add_space(16.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(locale.text("library")).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button(locale.text("refresh")).clicked() {
                            self.refresh();
                        }
                    });
                });

                let mut opened = None;
                let height = (ui.available_height() - 112.0).max(34.0);
                egui::ScrollArea::vertical()
                    .id_salt("files")
                    .max_height(height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for entry in self.library.iter().filter(|entry| entry.game == self.game) {
                            let active = self
                                .current
                                .as_ref()
                                .is_some_and(|save| save.path == entry.path);
                            let response = library_row(ui, entry, active, locale);
                            if response.clicked() && !active {
                                opened = Some(entry.path.clone());
                            }
                        }
                        if self.library.iter().all(|entry| entry.game != self.game) {
                            ui.label(RichText::new(locale.text("no_files")).color(MUTED));
                        }
                    });
                if let Some(path) = opened {
                    self.open(&path);
                }

                if ui
                    .add_sized(
                        [ui.available_width(), 34.0],
                        egui::Button::new(locale.text("open")),
                    )
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_file()
                {
                    self.open(&path);
                }
                if ui
                    .add_sized(
                        [ui.available_width(), 34.0],
                        egui::Button::new(locale.text("folder")),
                    )
                    .clicked()
                    && let Some(folder) = rfd::FileDialog::new().pick_folder()
                {
                    if !self.folders.contains(&folder) {
                        self.folders.push(folder);
                    }
                    self.refresh();
                }
            });
    }

    fn document(&mut self, ui: &mut egui::Ui) {
        let locale = self.locale;
        let Some(document) = self.current.as_mut() else {
            ui.add_space(50.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(self.game.title()).size(32.0).strong());
                ui.label(RichText::new(locale.text("choose")).color(MUTED));
                if let Some((_, message)) = &self.notice {
                    ui.label(RichText::new(message).color(ERROR));
                }
            });
            return;
        };
        let [accent, _, success] = logo_colors(document.game);
        let mut action = None;
        let pending = document.changed();
        document_banner(ui, document, locale);
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            let save_button = egui::Button::new(
                RichText::new(locale.text("save"))
                    .strong()
                    .color(if pending > 0 { BG } else { MUTED }),
            )
            .fill(if pending > 0 { accent } else { SURFACE });
            if ui.add_enabled(pending > 0, save_button).clicked() {
                action = Some("save");
            }
            if ui.button(locale.text("export")).clicked() {
                action = Some("export");
            }
            if pending > 0 {
                if ui.button(locale.text("discard")).clicked() {
                    action = Some("reload");
                }
                let text = locale.text("pending").replace("{n}", &pending.to_string());
                ui.label(RichText::new(text).color(WARM));
            } else if ui.button(locale.text("reload")).clicked() {
                action = Some("reload");
            }
        });
        ui.add_space(4.0);
        Frame::new()
            .fill(SIDEBAR)
            .corner_radius(10)
            .inner_margin(Margin::same(5))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (tab, label) in [
                        (Tab::City, locale.text("overview")),
                        (Tab::Collections, locale.text("collections")),
                        (Tab::Advanced, locale.text("advanced")),
                        (Tab::Backups, locale.text("backups")),
                    ] {
                        if ui.selectable_label(self.tab == tab, label).clicked() {
                            self.tab = tab;
                            self.query.clear();
                        }
                    }
                });
            });
        if let Some((failed, message)) = &self.notice {
            let color = if *failed { ERROR } else { success };
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(message).color(color));
            });
        }
        ui.add_space(8.0);
        if document.snapshots.len() > 1 {
            ui.horizontal(|ui| {
                ui.label(locale.text("version"));
                let label = |index: usize| {
                    let raw = &document.snapshots[index];
                    let (kind, number) = raw.split_once(' ').unwrap_or((raw, ""));
                    let kind = locale.text(if kind == "Manual" {
                        "manual"
                    } else {
                        "automatic"
                    });
                    if index == document.latest {
                        format!("{kind} {number} · {}", locale.text("latest"))
                    } else {
                        format!("{kind} {number}")
                    }
                };
                egui::ComboBox::from_id_salt("snapshot")
                    .selected_text(label(self.snapshot))
                    .show_ui(ui, |ui| {
                        for index in 0..document.snapshots.len() {
                            ui.selectable_value(&mut self.snapshot, index, label(index));
                        }
                    });
            });
        }
        if self.tab == Tab::Backups {
            match document.backups() {
                Ok(paths) if paths.is_empty() => {
                    ui.label(RichText::new(locale.text("backup_help")).color(MUTED));
                }
                Ok(paths) => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for path in paths {
                            ui.horizontal(|ui| {
                                if path.extension().is_some_and(|ext| ext == "bk") {
                                    ui.label(locale.text("game_backup"))
                                        .on_hover_text(save::file_label(&path));
                                } else {
                                    ui.label(save::file_label(&path));
                                }
                                if ui
                                    .add_enabled(
                                        pending == 0,
                                        egui::Button::new(locale.text("restore_open")),
                                    )
                                    .clicked()
                                {
                                    self.restore = Some(path);
                                }
                            });
                        }
                    });
                }
                Err(error) => {
                    ui.label(format!("{}\n{error:#}", locale.text("failed")));
                }
            }
        } else {
            let query = self.query.trim().to_lowercase();
            if self.tab == Tab::Collections {
                let available = Collection::ALL
                    .into_iter()
                    .filter(|option| {
                        document
                            .fields
                            .iter()
                            .any(|field| option.matches(&field.path))
                    })
                    .collect::<Vec<_>>();
                if !available.contains(&self.collection)
                    && let Some(first) = available.first()
                {
                    self.collection = *first;
                }
                if !available.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        egui::ComboBox::from_id_salt("collection")
                            .selected_text(locale.text(self.collection.label_key()))
                            .show_ui(ui, |ui| {
                                for option in available {
                                    if ui
                                        .selectable_value(
                                            &mut self.collection,
                                            option,
                                            locale.text(option.label_key()),
                                        )
                                        .changed()
                                    {
                                        self.query.clear();
                                    }
                                }
                            });
                        let bulk = match self.collection {
                            Collection::Affinity => {
                                Some((locale.text("max_affinity"), MAX_AFFINITY.to_string()))
                            }
                            Collection::Buildings => {
                                Some((locale.text("unlock_buildings"), "true".to_owned()))
                            }
                            Collection::Vehicles => {
                                Some((locale.text("unlock_cars"), "true".to_owned()))
                            }
                            Collection::Research => {
                                Some((locale.text("complete_research"), "true".to_owned()))
                            }
                            _ => None,
                        };
                        if let Some((label, value)) = bulk
                            && ui.button(label).clicked()
                        {
                            apply_collection(document, self.collection, &value);
                        }
                    });
                    if self.collection == Collection::Affinity {
                        ui.label(RichText::new(locale.text("affinity_help")).color(MUTED));
                    } else if self.collection == Collection::Buildings {
                        ui.label(RichText::new(locale.text("building_help")).color(MUTED));
                    } else if self.collection.is_resource_automation() {
                        ui.label(RichText::new(locale.text("auto_help")).color(MUTED));
                    }
                }
            }
            if self.tab != Tab::City {
                ui.add(
                    TextEdit::singleline(&mut self.query)
                        .hint_text(locale.text("search"))
                        .desired_width(275.0),
                );
            }
            let tab = self.tab;
            let game = document.game;
            let survival = document.is_pc2_city() && document.pc2_survival();
            egui::ScrollArea::vertical()
                .id_salt(("fields", tab as u8))
                .show(ui, |ui| {
                    if tab == Tab::City {
                        if document.is_pc2_city() {
                            city_mode_panel(ui, document, locale);
                        }
                        progression_panel(ui, document, self.snapshot, locale);
                    }
                    let mut shown = 0;
                    Frame::new()
                        .fill(SURFACE)
                        .stroke(Stroke::new(1.0_f32, BORDER))
                        .corner_radius(10)
                        .inner_margin(Margin::same(14))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            let label_width = (ui.available_width() * 0.53).clamp(180.0, 410.0);
                            let field_width =
                                (ui.available_width() - label_width - 30.0).clamp(98.0, 300.0);
                            ui.label(
                                RichText::new(if tab == Tab::City {
                                    locale.text("city_data")
                                } else {
                                    locale.text("fields")
                                })
                                .strong()
                                .size(16.0),
                            );
                            ui.add_space(3.0);
                            egui::Grid::new(("document-fields", tab as u8))
                                .num_columns(2)
                                .spacing([18.0, 5.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    for field in &mut document.fields {
                                        if field.snapshot != self.snapshot {
                                            continue;
                                        }
                                        let section = field_section(&field.path);
                                        let is_visible = match tab {
                                            Tab::City => section == Tab::City,
                                            Tab::Collections => {
                                                self.collection.matches(&field.path)
                                            }
                                            Tab::Advanced => true,
                                            Tab::Backups => false,
                                        };
                                        let label = field_name(&field.path, &field.label, locale);
                                        if !is_visible
                                            || (!query.is_empty()
                                                && !field.path.to_lowercase().contains(&query)
                                                && !label.to_lowercase().contains(&query))
                                        {
                                            continue;
                                        }
                                        shown += 1;
                                        let label_color =
                                            if field.changed() { accent } else { INK };
                                        let locked = field.locked(game)
                                            || (survival
                                                && save::PC2_SANDBOX_FLAGS
                                                    .contains(&field.path.as_str())
                                                && field.value == "false");
                                        let label = if locked
                                            && !Collection::Buildings.matches(&field.path)
                                        {
                                            format!("{label} · {}", locale.text("protected"))
                                        } else {
                                            label
                                        };
                                        let row = egui::Label::new(
                                            RichText::new(&label).color(label_color),
                                        )
                                        .truncate();
                                        let label_response = ui
                                            .allocate_ui_with_layout(
                                                egui::vec2(label_width, 30.0),
                                                Layout::left_to_right(Align::Center),
                                                |ui| {
                                                    ui.add(row).on_hover_text(format!(
                                                        "{label}\n{}",
                                                        field.path
                                                    ))
                                                },
                                            )
                                            .inner;
                                        ui.push_id((shown, &field.path), |ui| {
                                            ui.add_enabled_ui(!locked, |ui| {
                                                if field.kind == Kind::Boolean {
                                                    let mut checked = field.value == "true";
                                                    if ui
                                                        .checkbox(&mut checked, "")
                                                        .labelled_by(label_response.id)
                                                        .changed()
                                                    {
                                                        field.value = checked.to_string();
                                                    }
                                                } else if game == Game::Two
                                                    && let Some(options) =
                                                        field_options(&field.path, locale)
                                                {
                                                    let selected = options
                                                        .iter()
                                                        .find(|(value, _)| *value == field.value)
                                                        .map_or(
                                                            field.value.as_str(),
                                                            |(_, title)| *title,
                                                        );
                                                    egui::ComboBox::from_id_salt("field-options")
                                                        .selected_text(selected)
                                                        .show_ui(ui, |ui| {
                                                            for (value, name) in options {
                                                                ui.selectable_value(
                                                                    &mut field.value,
                                                                    value.to_owned(),
                                                                    name,
                                                                );
                                                            }
                                                        })
                                                        .response
                                                        .labelled_by(label_response.id);
                                                } else {
                                                    ui.add_sized(
                                                        [field_width, 27.0],
                                                        TextEdit::singleline(&mut field.value),
                                                    )
                                                    .labelled_by(label_response.id);
                                                }
                                            });
                                        });
                                        ui.end_row();
                                    }
                                });
                            if shown == 0 {
                                ui.label(RichText::new(locale.text("empty_fields")).color(MUTED));
                            }
                        });
                });
        }

        let result = match action {
            Some("save") => Some(
                document
                    .save()
                    .map(|updated| (Some(updated), locale.text("saved"))),
            ),
            Some("reload") => Some(
                Save::open(&document.path).map(|updated| (Some(updated), locale.text("reloaded"))),
            ),
            Some("export") => rfd::FileDialog::new()
                .set_file_name(save::file_label(&document.path))
                .save_file()
                .map(|path| {
                    document
                        .export(&path)
                        .map(|()| (None, locale.text("exported")))
                }),
            _ => None,
        };
        match result {
            Some(Ok((updated, message))) => {
                if let Some(updated) = updated {
                    self.snapshot = self.snapshot.min(updated.snapshots.len() - 1);
                    self.remember(&updated);
                    self.current = Some(updated);
                }
                self.status(false, message);
            }
            Some(Err(error)) => {
                self.status(true, format!("{}\n{error:#}", self.locale.text("failed")))
            }
            None => {}
        }
    }
}

impl eframe::App for Editor {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("locale", self.locale.code().to_owned());
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let locale = self.locale;
        let [accent, _, _] = logo_colors(self.game);
        if ctx.style().visuals.selection.stroke.color != accent {
            let mut style = (*ctx.style()).clone();
            style.visuals.selection.bg_fill = match self.game {
                Game::One => Color32::from_rgb(50, 62, 108),
                Game::Two => Color32::from_rgb(42, 70, 105),
            };
            style.visuals.selection.stroke.color = accent;
            style.visuals.widgets.active.bg_stroke.color = accent;
            ctx.set_style(style);
        }
        if ctx.input(|i| i.viewport().close_requested())
            && self
                .current
                .as_ref()
                .is_some_and(|save| save.changed() != 0)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.status(true, locale.text("pending_guard"));
        }
        for path in ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .take(1)
                .collect::<Vec<_>>()
        }) {
            self.open(&path);
        }
        self.sidebar(ctx);
        egui::CentralPanel::default()
            .frame(Frame::new().fill(BG).inner_margin(Margin::same(26)))
            .show(ctx, |ui| self.document(ui));
        if let Some(backup) = self.restore.clone() {
            egui::Window::new(locale.text("restore_backup"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(locale.text("restore_help"));
                    ui.horizontal(|ui| {
                        if ui.button(locale.text("cancel")).clicked() {
                            self.restore = None;
                        }
                        if ui.button(locale.text("restore")).clicked() {
                            if let Some(document) = &self.current {
                                match document.restore(&backup) {
                                    Ok(updated) => {
                                        self.snapshot = updated.latest;
                                        self.remember(&updated);
                                        self.current = Some(updated);
                                        self.status(false, locale.text("restored"));
                                    }
                                    Err(error) => self.status(
                                        true,
                                        format!("{}\n{error:#}", self.locale.text("failed")),
                                    ),
                                }
                            }
                            self.restore = None;
                        }
                    });
                });
        }
    }
}

fn library_row(ui: &mut egui::Ui, entry: &Entry, selected: bool, locale: Locale) -> egui::Response {
    let title = settings_title(&entry.path, locale).unwrap_or(&entry.title);
    let [accent, _, _] = logo_colors(entry.game);
    let subtitle = locale.text(archive_key(&entry.path));
    let (_, rect) = ui.allocate_space(egui::vec2(ui.available_width(), 49.0));
    let response = ui.interact(
        rect,
        ui.make_persistent_id(&entry.path),
        egui::Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            format!("{title}, {subtitle}"),
        )
    });

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let fill = if selected {
            HEADER
        } else if response.hovered() {
            SURFACE
        } else {
            Color32::TRANSPARENT
        };
        let stroke = if response.has_focus() {
            Stroke::new(1.0_f32, accent)
        } else if selected {
            Stroke::new(1.0_f32, BORDER)
        } else {
            Stroke::NONE
        };
        painter.rect(rect, 8.0, fill, stroke, egui::StrokeKind::Inside);
        for (text, font_size, color, top) in [
            (title, 15.0, if selected { accent } else { INK }, 6.0),
            (subtitle, 12.0, MUTED, 27.0),
        ] {
            let mut job = egui::text::LayoutJob::simple_singleline(
                text.to_owned(),
                egui::FontId::proportional(font_size),
                color,
            );
            job.wrap.max_width = (rect.width() - 24.0).max(0.0);
            job.wrap.max_rows = 1;
            painter.galley(
                rect.left_top() + egui::vec2(12.0, top),
                painter.layout_job(job),
                color,
            );
        }
    }

    response.on_hover_text(format!("{title}\n{}", entry.path.display()))
}

fn archive_key(path: &Path) -> &'static str {
    let name = save::file_label(path);
    if name.contains("_auto_") {
        "auto_archive"
    } else if name.contains("_manual_") {
        "manual_archive"
    } else if name.contains("settings") || name == "_user.save" {
        "global"
    } else {
        "main"
    }
}

fn settings_title(path: &Path, locale: Locale) -> Option<&'static str> {
    locale.get(match save::file_label(path).as_str() {
        "global_settings" => "game_settings",
        "survival_global_settings" => "survival_settings",
        "os_settings" => "system_settings",
        "_user.save" => "player_settings",
        _ => return None,
    })
}

fn document_banner(ui: &mut egui::Ui, document: &Save, locale: Locale) {
    let wide = ui.available_width() >= 560.0;
    let colors = logo_colors(document.game);
    let settings = settings_title(&document.path, locale);
    let title = settings.unwrap_or(&document.title);
    let header = Frame::new()
        .fill(HEADER)
        .corner_radius(14)
        .inner_margin(Margin::symmetric(19, 14))
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.set_min_width(width);
            ui.allocate_ui_with_layout(
                egui::vec2(width - if wide { 172.0 } else { 0.0 }, 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{}  /  {}",
                            document.game.title(),
                            locale.text(archive_key(&document.path))
                        ))
                        .color(colors[0])
                        .size(13.0),
                    );
                    ui.add(egui::Label::new(RichText::new(title).size(31.0).strong()).truncate())
                        .on_hover_text(document.path.display().to_string());
                    if settings.is_none() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(save::file_label(&document.path))
                                    .color(MUTED)
                                    .size(12.0),
                            )
                            .truncate(),
                        )
                        .on_hover_text(document.path.display().to_string());
                    }
                },
            );
        });
    if wide {
        draw_city_plan(ui.painter(), header.response.rect, colors);
    }
}

fn draw_city_plan(painter: &egui::Painter, banner: egui::Rect, colors: [Color32; 3]) {
    let origin = banner.right_top() + egui::vec2(-172.0, banner.height() * 0.55);
    let plan = |column: f32, row: f32, floor: f32| {
        origin + egui::vec2(15.0 * (column + row), 7.5 * (column - row) - floor)
    };
    let grid = Stroke::new(1.0_f32, BORDER);
    for lane in 0..=5 {
        let lane = lane as f32;
        painter.line_segment([plan(lane, 0.0, 0.0), plan(lane, 5.0, 0.0)], grid);
        painter.line_segment([plan(0.0, lane, 0.0), plan(5.0, lane, 0.0)], grid);
    }
    for ((column, row, height), color) in [
        (0.7, 2.7, 12.0),
        (2.1, 2.1, 25.0),
        (3.5, 1.1, 16.0),
        (2.9, 3.6, 32.0),
    ]
    .into_iter()
    .zip(colors.into_iter().cycle())
    {
        let outline = Stroke::new(1.0_f32, color.gamma_multiply(0.65));
        let points = [
            (column, row),
            (column + 0.8, row),
            (column + 0.8, row + 0.8),
            (column, row + 0.8),
        ];
        let roof = points
            .iter()
            .map(|&(x, y)| plan(x, y, height))
            .collect::<Vec<_>>();
        painter.add(egui::Shape::convex_polygon(
            roof,
            color.gamma_multiply(0.2),
            outline,
        ));
        for &(x, y) in &points[..3] {
            painter.line_segment([plan(x, y, 0.0), plan(x, y, height)], outline);
        }
    }
}

fn progression_panel(ui: &mut egui::Ui, document: &mut Save, selected: usize, locale: Locale) {
    let (level_key, xp_key) = document.game.progression();
    let level = document
        .fields
        .iter()
        .find(|field| field.snapshot == selected && field.path == level_key)
        .and_then(|field| field.value.parse::<i64>().ok());
    let Some(level) = level else { return };
    let Some(xp) = document
        .fields
        .iter_mut()
        .find(|field| field.snapshot == selected && field.path == xp_key)
    else {
        return;
    };
    let Some(threshold) = document.game.next_level_xp(level) else {
        return;
    };
    let amount = xp.value.parse::<i64>().unwrap_or(0);
    Frame::new()
        .fill(SIDEBAR)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(12)
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} {level}", locale.text("level")))
                        .strong()
                        .size(18.0),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_enabled(amount != 0, egui::Button::new(locale.text("reset_xp")))
                        .clicked()
                    {
                        xp.value = "0".to_owned();
                    }
                });
            });
            ui.add(
                egui::ProgressBar::new((amount.max(0) as f32 / threshold as f32).clamp(0.0, 1.0))
                    .fill(logo_colors(document.game)[0])
                    .desired_height(5.0)
                    .corner_radius(3),
            );
            ui.label(
                RichText::new(
                    locale
                        .text("xp_progress")
                        .replace("{xp}", &amount.to_string())
                        .replace("{total}", &threshold.to_string()),
                )
                .color(MUTED)
                .size(12.0),
            );
        });
    ui.add_space(5.0);
}

fn apply_collection(document: &mut Save, collection: Collection, value: &str) {
    for &(id, _) in collection.catalog() {
        let path = format!("{}.{id}", collection.key());
        if let Some(field) = document.fields.iter_mut().find(|field| field.path == path)
            && !field.locked(document.game)
        {
            field.value = value.to_owned();
        }
    }
}

fn city_mode_panel(ui: &mut egui::Ui, document: &Save, locale: Locale) {
    let sandbox = document.pc2_sandbox();
    let survival = document.pc2_survival();
    let (label, color) = if survival {
        ("survival_city", POCKET_CITY_2_COLORS[2])
    } else if sandbox {
        ("sandbox", WARM)
    } else {
        ("regular_city", POCKET_CITY_2_COLORS[0])
    };
    Frame::new()
        .fill(SIDEBAR)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(locale.text(label)).strong().color(color));
            if sandbox {
                ui.label(
                    RichText::new(locale.text("sandbox_effects"))
                        .color(MUTED)
                        .size(13.0),
                );
            } else if !survival {
                ui.label(
                    RichText::new(locale.text("mode_help"))
                        .color(MUTED)
                        .size(13.0),
                );
            }
        });
    ui.add_space(4.0);
}

fn field_section(path: &str) -> Tab {
    let key = path.rsplit('.').next().unwrap_or(path);
    if matches!(
        key,
        "name"
            | "cityName"
            | "money"
            | "cash"
            | "level"
            | "XP"
            | "xp"
            | "exp"
            | "day"
            | "dayProgress"
            | "researchPoints"
            | "population"
            | "loanTaken"
            | "randomDisasters"
            | "survivalLifelines"
            | "incomeTax"
            | "propertyTax"
            | "salesTax"
            | "difficulty"
            | "infiniteMoney"
            | "maxLevel"
            | "unlockAll"
            | "disableQuests"
            | "isSandbox"
            | "sandboxWithMoney"
            | "totalUpgradePoints"
            | "disableUnspentUpgradeToCash"
    ) || !path.contains('.') && (key.starts_with("bestDay") || key.starts_with("highest"))
    {
        Tab::City
    } else {
        Tab::Advanced
    }
}

fn difficulty(id: i32, locale: Locale) -> Option<&'static str> {
    ["default", "easy", "hard", "expert"]
        .get(usize::try_from(id).ok()?)
        .and_then(|key| locale.get(key))
}

fn field_options(path: &str, locale: Locale) -> Option<Vec<(&'static str, &'static str)>> {
    Some(match path {
        "difficulty" => vec![
            ("0", locale.get("default")?),
            ("1", locale.get("easy")?),
            ("2", locale.get("hard")?),
            ("3", locale.get("expert")?),
        ],
        "lastDifficultySelection" => vec![
            ("0", locale.get("easy")?),
            ("1", locale.get("hard")?),
            ("2", locale.get("expert")?),
        ],
        "controllerLayout" => vec![
            ("0", locale.get("default")?),
            ("1", "A"),
            ("2", "B"),
            ("3", "C"),
        ],
        "desiredLang" => Locale::ALL
            .into_iter()
            .map(|(code, name)| {
                (
                    match code {
                        "pt-BR" => "pt-br",
                        "es-ES" => "es-es",
                        "es-419" => "es-mx",
                        "zh-Hans" => "zh",
                        "uk" => "uk-ua",
                        other => other,
                    },
                    name,
                )
            })
            .chain([("tr", "Türkçe")])
            .collect(),
        _ => return None,
    })
}

fn field_name(path: &str, fallback: &str, locale: Locale) -> String {
    if let Some((group, id)) = path.split_once('.')
        && let Ok(id) = id.parse::<i32>()
    {
        match group {
            "maxLevelReached" | "maxPopulationReached" => {
                let key = if group == "maxLevelReached" {
                    "max_level_record"
                } else {
                    "max_pop_record"
                };
                return format!(
                    "{} · {}",
                    locale.text(key),
                    difficulty(id, locale).unwrap_or(fallback)
                );
            }
            "resources" | "resourceAutoSellKeepAmt" | "resourceAutoConvertKeepAmt" => {
                return locale
                    .get(&format!("resource_{id}"))
                    .unwrap_or(fallback)
                    .to_owned();
            }
            "relationships" | "pendingGifts" => {
                return locale
                    .get(&format!("npc_{id}"))
                    .unwrap_or(fallback)
                    .to_owned();
            }
            "upgradesSpent" | "upgradesSpentCity" => {
                return locale
                    .get(&format!("upgrade_{id}"))
                    .unwrap_or(fallback)
                    .to_owned();
            }
            "unlockedCars" => {
                return format!("{} {:02}", locale.text("vehicle"), i64::from(id) + 1);
            }
            _ => {}
        }
    }
    let key = path.rsplit('.').next().unwrap_or(path);
    for (prefix, label) in [
        ("bestDayReached", "best_day"),
        ("highestStarsReached", "best_stars"),
        ("highestPopulationReached", "max_pop_record"),
    ] {
        if let Some(suffix) = key.strip_prefix(prefix) {
            let difficulty = match suffix {
                "Easy" => Some("easy"),
                "Hard" => Some("hard"),
                "Expert" => Some("expert"),
                _ => None,
            };
            if let Some(difficulty) = difficulty {
                return format!("{} · {}", locale.text(label), locale.text(difficulty));
            }
        }
    }
    let key = match key {
        "name" | "cityName" => "name",
        "money" | "cash" => "money",
        "XP" | "xp" | "exp" => "xp",
        "dayProgress" => "day_progress",
        "researchPoints" => "research_points",
        "incomeTax" => "income_tax",
        "propertyTax" => "property_tax",
        "salesTax" => "sales_tax",
        "infiniteMoney" => "infinite_money",
        "maxLevel" => "max_level",
        "unlockAll" => "unlock_all",
        "disableQuests" => "disable_quests",
        "isSandbox" => "sandbox",
        "sandboxWithMoney" => "sandbox_budget",
        "totalUpgradePoints" => "upgrade_points",
        "loanTaken" => "loan",
        "randomDisasters" => "disasters",
        "survivalLifelines" => "lifelines",
        "size" | "mapSize" => "map_size",
        "creationDate" => "created",
        "elapsedTime" => "playtime",
        "isSurvivalMode" => "survival_city",
        _ => key,
    };
    locale.get(key).unwrap_or(fallback).to_owned()
}
