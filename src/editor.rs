use crate::{
    catalog::{Collection, MAX_AFFINITY},
    locale::Locale,
    save::{self, Game, Kind, Save},
};
use gpui_kit::{base::StyledExt, prelude::FluentBuilder};
use gpui_kit::{
    component::{
        ActiveTheme, Disableable, IconName, Sizable,
        alert::Alert,
        button::{Button, ButtonVariants},
        checkbox::Checkbox,
        form::{self, h_form},
        group_box::{GroupBox, GroupBoxVariants},
        input::{Input, InputEvent, InputState},
        label::Label,
        menu::{DropdownMenu, PopupMenuItem},
        progress::Progress as ProgressBar,
        sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
        slider::{Slider, SliderEvent, SliderState},
        tab::{Tab as KitTab, TabBar},
        toolbar::Toolbar,
    },
    *,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

mod progression;

const BG: u32 = 0x12161b;
const BORDER: u32 = 0x323c47;
const INK: u32 = 0xf4f4f5;
const MUTED: u32 = 0xa7b1bf;
const PAGE_SIZE: usize = 32;

fn accent(game: Game) -> u32 {
    match game {
        Game::One => 0xffc65d,
        Game::Two => 0x64cef2,
    }
}

struct Entry {
    path: PathBuf,
    title: String,
    game: Game,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    City,
    Collections,
    Progress,
    Advanced,
    Backups,
}

pub struct Editor {
    game: Game,
    library: Vec<Entry>,
    folders: Vec<PathBuf>,
    current: Option<Save>,
    tab: Tab,
    notice: Option<(bool, String)>,
    restore: Option<PathBuf>,
    snapshot: usize,
    collection: Collection,
    locale: Locale,
    section: usize,
    page: usize,
    query: Entity<InputState>,
    inputs: HashMap<usize, (Entity<InputState>, Subscription)>,
    sliders: HashMap<SliderKey, (Entity<SliderState>, Subscription)>,
    _search_subscription: Subscription,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SliderKey {
    Tax(usize),
    Pet(i32),
}

fn locale_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("Pocket City Save Editor").join("language"))
}

impl Editor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let locale = locale_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|code| Locale::from_code(code.trim()))
            .unwrap_or_default();
        let query =
            cx.new(|cx| InputState::new(window, cx).placeholder(locale.text("search").to_owned()));
        let search_subscription = cx.subscribe(&query, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.page = 0;
                cx.notify();
            }
        });
        let mut editor = Self {
            game: Game::One,
            library: Vec::new(),
            folders: save::default_folders()
                .into_iter()
                .filter(|p| p.is_dir())
                .collect(),
            current: None,
            tab: Tab::City,
            notice: None,
            restore: None,
            snapshot: 0,
            collection: Collection::Affinity,
            locale,
            section: 0,
            page: 0,
            query,
            inputs: HashMap::new(),
            sliders: HashMap::new(),
            _search_subscription: search_subscription,
        };
        editor.refresh();
        let initial = std::env::args_os().nth(1).map(PathBuf::from).or_else(|| {
            editor
                .library
                .iter()
                .filter(|entry| archive_key(&entry.path) == "main")
                .max_by_key(|entry| {
                    std::fs::metadata(&entry.path)
                        .and_then(|meta| meta.modified())
                        .ok()
                })
                .map(|entry| entry.path.clone())
        });
        if let Some(path) = initial {
            editor.open(&path);
        }
        editor
    }

    fn refresh(&mut self) {
        self.library.retain(|entry| entry.path.is_file());
        for folder in self.folders.clone() {
            match save::scan(&folder) {
                Ok(paths) => {
                    for path in paths {
                        if let Ok(document) = Save::open(&path) {
                            self.remember(&document);
                        }
                    }
                }
                Err(error) => self.failure(error),
            }
        }
        self.library
            .sort_by_cached_key(|entry| (entry.title.to_lowercase(), entry.path.clone()));
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

    fn reset_controls(&mut self) {
        self.sliders.clear();
        self.inputs.clear();
        self.page = 0;
        self.restore = None;
    }

    fn failure(&mut self, error: impl std::fmt::Display) {
        self.notice = Some((true, format!("{}\n{error:#}", self.locale.text("failed"))));
    }

    fn open(&mut self, path: &Path) {
        if self
            .current
            .as_ref()
            .is_some_and(|save| save.changed() != 0)
        {
            self.notice = Some((true, self.locale.text("pending_guard").to_owned()));
            return;
        }
        match Save::open(path) {
            Ok(document) => {
                self.snapshot = document.latest;
                self.game = document.game;
                self.section = 0;
                self.tab = if document
                    .fields
                    .iter()
                    .any(|field| field_section(&field.path) == Tab::City)
                {
                    Tab::City
                } else {
                    Tab::Advanced
                };
                self.reset_controls();
                self.remember(&document);
                self.current = Some(document);
                self.notice = None;
            }
            Err(error) => self.failure(error),
        }
    }

    fn settle(&mut self, result: anyhow::Result<Save>, key: &str) {
        match result {
            Ok(save) => {
                self.snapshot = self.snapshot.min(save.snapshots.len() - 1);
                self.reset_controls();
                self.remember(&save);
                self.current = Some(save);
                self.notice = Some((false, self.locale.text(key).to_owned()));
            }
            Err(error) => self.failure(error),
        }
    }

    fn set_language(&mut self, code: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.locale = Locale::from_code(code);
        self.query.update(cx, |state, cx| {
            state.set_placeholder(self.locale.text("search").to_owned(), window, cx);
        });
        if let Some(path) = locale_path()
            && let Some(dir) = path.parent()
            && let Err(error) =
                std::fs::create_dir_all(dir).and_then(|()| std::fs::write(path, code))
        {
            self.failure(error);
        }
    }

    fn open_from_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.open(&path);
            if self.notice.is_none() {
                self.query
                    .update(cx, |state, cx| state.set_value("", window, cx));
            }
        }
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let locale = self.locale;
        let header = SidebarHeader::new().child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .w_full()
                .child(
                    div()
                        .w_8()
                        .h_8()
                        .rounded_md()
                        .bg(rgb(accent(self.game)))
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_bold()
                        .child(Label::new("PC").text_color(rgb(BG))),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(div().font_semibold().child(Label::new("Pocket City")))
                        .child(subtle(locale.text("app"))),
                ),
        );
        let games = TabBar::new("game-selector")
            .segmented()
            .small()
            .selected_index(usize::from(self.game == Game::Two))
            .children(
                [Game::One, Game::Two]
                    .into_iter()
                    .map(|game| KitTab::new().label(game.title())),
            )
            .on_click(cx.listener(|this, index: &usize, _, cx| {
                this.game = if *index == 0 { Game::One } else { Game::Two };
                cx.notify();
            }));
        let mut groups = Vec::new();
        for kind in ["main", "auto_archive", "manual_archive", "global"] {
            let items = self
                .library
                .iter()
                .filter(|entry| entry.game == self.game && archive_key(&entry.path) == kind)
                .map(|entry| {
                    let path = entry.path.clone();
                    let selected = self.current.as_ref().is_some_and(|doc| doc.path == path);
                    let title = settings_title(&path, locale)
                        .unwrap_or(&entry.title)
                        .to_owned();
                    SidebarMenuItem::new(title.clone())
                        .icon(if kind == "main" {
                            IconName::BookOpen
                        } else {
                            IconName::Inbox
                        })
                        .accessibility_label(format!("{title}, {}", locale.text(kind)))
                        .active(selected)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.open(&path);
                            if this.notice.is_none() {
                                this.query
                                    .update(cx, |state, cx| state.set_value("", window, cx));
                            }
                            cx.notify();
                        }))
                })
                .collect::<Vec<_>>();
            if !items.is_empty() {
                groups.push(
                    SidebarGroup::new(locale.text(kind)).child(SidebarMenu::new().children(items)),
                );
            }
        }
        let editor = cx.weak_entity();
        let language = library_action("language", locale.name(), IconName::Globe, true)
            .accessibility_label(format!("{}, {}", locale.text("language"), locale.name()))
            .dropdown_menu(move |menu, _, _| {
                Locale::ALL.into_iter().fold(menu, |menu, (code, name)| {
                    let editor = editor.clone();
                    menu.item(
                        PopupMenuItem::new(name)
                            .checked(locale.code() == code)
                            .on_click(move |_, window, cx| {
                                let _ = editor.update(cx, |this, cx| {
                                    this.set_language(code, window, cx);
                                    cx.notify();
                                });
                            }),
                    )
                })
            });
        let footer = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_1()
            .border_t_1()
            .border_color(rgb(BORDER))
            .pt_2()
            .child(
                library_action("open", locale.text("open"), IconName::File, false).on_click(
                    cx.listener(|this, _, window, cx| {
                        this.open_from_dialog(window, cx);
                        cx.notify();
                    }),
                ),
            )
            .child(
                library_action("folder", locale.text("folder"), IconName::FolderOpen, false)
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder()
                            && !this.folders.contains(&folder)
                        {
                            this.folders.push(folder);
                            this.refresh();
                            cx.notify();
                        }
                    })),
            )
            .child(language);
        Sidebar::new("save-library")
            .w(px(280.0))
            .collapsible(false)
            .header(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .gap_3()
                    .child(header)
                    .child(games)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(subtle(locale.text("library")))
                            .child(
                                action(
                                    cx,
                                    "refresh",
                                    locale.text("refresh"),
                                    false,
                                    true,
                                    |this, _, _| this.refresh(),
                                )
                                .ghost(),
                            ),
                    )
                    .when(groups.is_empty(), |header| {
                        header.child(subtle(locale.text("no_files")))
                    }),
            )
            .children(groups)
            .footer(footer)
    }

    fn document(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let locale = self.locale;
        let Some(document) = self.current.as_ref() else {
            return div()
                .flex()
                .flex_col()
                .items_start()
                .justify_center()
                .gap_3()
                .flex_1()
                .p_8()
                .child(Label::new(self.game.title()).text_3xl().font_semibold())
                .child(subtle(locale.text("choose")))
                .child(action(
                    cx,
                    "empty-open",
                    locale.text("open"),
                    true,
                    true,
                    Self::open_from_dialog,
                ))
                .when_some(self.notice.as_ref(), |view, (_, message)| {
                    view.child(subtle(message.clone()))
                });
        };
        let pending = document.changed();
        let game = document.game;
        let archive_help = matches!(
            archive_key(&document.path),
            "auto_archive" | "manual_archive"
        ) && !document.pc2_survival();
        let has_progress = document.progress[self.snapshot].is_some();
        let heading = div()
            .flex()
            .flex_col()
            .gap_1()
            .min_w_0()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().size_2().rounded_full().bg(rgb(accent(game))))
                    .child(subtle(format!(
                        "{} · {}",
                        game.title(),
                        locale.text(archive_key(&document.path))
                    ))),
            )
            .child(
                div().text_2xl().font_semibold().child(Label::new(
                    settings_title(&document.path, locale)
                        .unwrap_or(&document.title)
                        .to_owned(),
                )),
            );
        let toolbar = Toolbar::new("document-actions")
            .child(
                action(
                    cx,
                    "reload",
                    locale.text(if pending > 0 { "discard" } else { "reload" }),
                    false,
                    true,
                    |this, _, _| {
                        if let Some(doc) = &this.current {
                            this.settle(Save::open(&doc.path), "reloaded");
                        }
                    },
                )
                .ghost(),
            )
            .child(action(
                cx,
                "export",
                locale.text("export"),
                false,
                true,
                |this, _, _| {
                    if let Some(doc) = &this.current
                        && let Some(path) = rfd::FileDialog::new()
                            .set_file_name(save::file_label(&doc.path))
                            .save_file()
                    {
                        match doc.export(&path) {
                            Ok(()) => {
                                this.notice = Some((false, this.locale.text("exported").to_owned()))
                            }
                            Err(error) => this.failure(error),
                        }
                    }
                },
            ))
            .child(action(
                cx,
                "save",
                locale.text("save"),
                true,
                pending != 0,
                |this, _, _| {
                    if let Some(document) = &this.current {
                        this.settle(document.save(), "saved");
                    }
                },
            ));
        let header = div()
            .flex()
            .flex_wrap()
            .items_end()
            .justify_between()
            .gap_3()
            .child(heading)
            .child(toolbar);
        let choices = [
            (Tab::City, "overview"),
            (Tab::Collections, "collections"),
            (Tab::Progress, "progression_tab"),
            (Tab::Advanced, "advanced"),
            (Tab::Backups, "backups"),
        ]
        .into_iter()
        .filter(|(tab, _)| *tab != Tab::Progress || has_progress)
        .collect::<Vec<_>>();
        let tabs = TabBar::new("document-tabs")
            .underline()
            .selected_index(
                choices
                    .iter()
                    .position(|(tab, _)| *tab == self.tab)
                    .unwrap_or_default(),
            )
            .children(
                choices
                    .iter()
                    .map(|&(_, label)| KitTab::new().label(locale.text(label))),
            )
            .on_click(cx.listener(move |this, index: &usize, window, cx| {
                if let Some(&(tab, _)) = choices.get(*index) {
                    this.tab = tab;
                    this.page = 0;
                    this.query
                        .update(cx, |state, cx| state.set_value("", window, cx));
                    cx.notify();
                }
            }));

        let snapshots = if document.snapshots.len() > 1 {
            let label = |index: usize| {
                let (kind, number) = document.snapshots[index]
                    .split_once(' ')
                    .unwrap_or(("", ""));
                format!(
                    "{} {}{}",
                    locale.text(if kind == "Manual" {
                        "manual"
                    } else {
                        "automatic"
                    }),
                    number,
                    if index == document.latest {
                        format!(" · {}", locale.text("latest"))
                    } else {
                        String::new()
                    }
                )
            };
            let editor = cx.weak_entity();
            let selected = self.snapshot;
            let options = (0..document.snapshots.len()).map(label).collect::<Vec<_>>();
            Some(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(subtle(locale.text("version")))
                    .child(
                        Button::new("snapshot")
                            .label(options[selected].clone())
                            .icon(IconName::ChevronDown)
                            .outline()
                            .small()
                            .dropdown_menu(move |menu, _, _| {
                                options
                                    .iter()
                                    .enumerate()
                                    .fold(menu, |menu, (index, label)| {
                                        let editor = editor.clone();
                                        menu.item(
                                            PopupMenuItem::new(label.clone())
                                                .checked(index == selected)
                                                .on_click(move |_, _, cx| {
                                                    let _ = editor.update(cx, |this, cx| {
                                                        this.snapshot = index;
                                                        this.reset_controls();
                                                        cx.notify();
                                                    });
                                                }),
                                        )
                                    })
                            }),
                    ),
            )
        } else {
            None
        };
        let body = match self.tab {
            Tab::Backups => self.backup_view(cx),
            Tab::Progress => self.progression(window, cx),
            _ => self.fields(window, cx),
        };
        div()
            .flex()
            .flex_col()
            .min_w_0()
            .gap_3()
            .flex_1()
            .h_full()
            .p_5()
            .child(header)
            .when(archive_help, |root| {
                root.child(subtle(locale.text("archive_edit_help")))
            })
            .child(tabs)
            .when(pending > 0, |root| {
                root.child(subtle(
                    locale.text("pending").replace("{n}", &pending.to_string()),
                ))
            })
            .when_some(snapshots, |root, row| root.child(row))
            .when_some(self.notice.as_ref(), |root, (error, message)| {
                root.child(if *error {
                    Alert::error("save-notice", message.clone())
                } else {
                    Alert::success("save-notice", message.clone())
                })
            })
            .child(
                div()
                    .id(("document-scroll", self.tab as usize))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(body),
            )
    }

    fn backup_view(&self, cx: &mut Context<Self>) -> Div {
        let locale = self.locale;
        let document = self.current.as_ref().unwrap();
        let mut view = column();
        match document.backups() {
            Ok(paths) => {
                if paths.is_empty() {
                    view = view.child(subtle(locale.text("backup_help")));
                }
                for (index, path) in paths.into_iter().enumerate() {
                    let name = if path.extension().is_some_and(|ext| ext == "bk") {
                        locale.text("game_backup").to_owned()
                    } else {
                        save::file_label(&path)
                    };
                    view = view.child(row(&name, "").child(action(
                        cx,
                        ("restore", index),
                        locale.text("restore_open"),
                        false,
                        document.changed() == 0,
                        move |this, _, _| this.restore = Some(path.clone()),
                    )));
                }
            }
            Err(error) => view = view.child(subtle(error.to_string())),
        }
        if let Some(path) = &self.restore {
            let path = path.clone();
            view = view.child(
                row(locale.text("restore_backup"), locale.text("restore_help")).child(
                    div()
                        .flex()
                        .gap_2()
                        .child(action(
                            cx,
                            "restore-confirm",
                            locale.text("restore"),
                            true,
                            true,
                            move |this, _, _| {
                                if let Some(document) = &this.current {
                                    this.settle(document.restore(&path), "restored");
                                }
                            },
                        ))
                        .child(action(
                            cx,
                            "restore-cancel",
                            locale.text("cancel"),
                            false,
                            true,
                            |this, _, _| this.restore = None,
                        )),
                ),
            );
        }
        view
    }

    fn fields(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let locale = self.locale;
        let document = self.current.as_ref().unwrap();
        let query = self.query.read(cx).value().trim().to_lowercase();
        let mut view = column();
        if self.tab == Tab::City {
            view = view.child(self.city_summary(cx));
        }
        if self.tab == Tab::Collections {
            let available = Collection::ALL
                .into_iter()
                .filter(|choice| {
                    document
                        .fields
                        .iter()
                        .any(|field| field.snapshot == self.snapshot && choice.matches(&field.path))
                })
                .collect::<Vec<_>>();
            if !available.contains(&self.collection)
                && let Some(&first) = available.first()
            {
                self.collection = first;
            }
            let editor = cx.weak_entity();
            let selected = self.collection;
            let mut controls = div()
                .flex()
                .items_center()
                .flex_wrap()
                .gap_2()
                .child(
                    Button::new("collections")
                        .label(locale.text(selected.label_key()))
                        .icon(IconName::ChevronDown)
                        .outline()
                        .small()
                        .disabled(available.is_empty())
                        .dropdown_menu(move |menu, _, _| {
                            available.iter().copied().fold(menu, |menu, collection| {
                                let editor = editor.clone();
                                menu.item(
                                    PopupMenuItem::new(locale.text(collection.label_key()))
                                        .checked(selected == collection)
                                        .on_click(move |_, _, cx| {
                                            let _ = editor.update(cx, |this, cx| {
                                                this.collection = collection;
                                                this.page = 0;
                                                cx.notify();
                                            });
                                        }),
                                )
                            })
                        }),
                )
                .child(Input::new(&self.query).w(px(240.0)));
            let bulk = match self.collection {
                Collection::Affinity => {
                    Some(("max_affinity", MAX_AFFINITY.to_string(), "affinity_help"))
                }
                Collection::Buildings => {
                    Some(("unlock_buildings", "true".to_owned(), "building_help"))
                }
                Collection::Vehicles => Some(("unlock_cars", "true".to_owned(), "")),
                _ => None,
            };
            let helper = if let Some((label, value, helper)) = bulk {
                let collection = self.collection;
                controls = controls.child(action(
                    cx,
                    "collection-bulk",
                    locale.text(label),
                    false,
                    true,
                    move |this, _, _| {
                        if let Some(doc) = &mut this.current {
                            for field in &mut doc.fields {
                                if field.snapshot == this.snapshot
                                    && collection.matches(&field.path)
                                    && !field.locked(doc.game)
                                {
                                    field.value.clone_from(&value);
                                }
                            }
                        }
                        this.inputs.clear();
                    },
                ));
                helper
            } else if self.collection.is_resource_automation() {
                "auto_help"
            } else if self.collection == Collection::GlobalFlags {
                "game_records_help"
            } else {
                ""
            };
            view = view.child(controls).when(!helper.is_empty(), |view| {
                view.child(subtle(locale.text(helper)))
            });
        } else if self.tab == Tab::Advanced {
            view = view.child(Input::new(&self.query).w(px(300.0)));
        }
        let mut entries = Vec::new();
        for (index, field) in document.fields.iter().enumerate() {
            let visible = match self.tab {
                Tab::City => field_section(&field.path) == Tab::City,
                Tab::Collections => self.collection.matches(&field.path),
                _ => true,
            };
            if field.snapshot != self.snapshot || !visible {
                continue;
            }
            let name = field_name(&field.path, &field.label, locale);
            if !query.is_empty()
                && !name.to_lowercase().contains(&query)
                && !field.path.to_lowercase().contains(&query)
            {
                continue;
            }
            entries.push((index, name));
        }
        if entries.is_empty() {
            return view.child(subtle(locale.text("empty_fields")));
        }
        let page_count = entries.len().div_ceil(PAGE_SIZE);
        self.page = self.page.min(page_count - 1);
        if page_count > 1 {
            let page = self.page;
            view = view.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(action(
                        cx,
                        "previous",
                        "‹",
                        false,
                        page > 0,
                        |this, _, _| this.page -= 1,
                    ))
                    .child(subtle(format!("{} / {page_count}", page + 1)))
                    .child(action(
                        cx,
                        "next",
                        "›",
                        false,
                        page + 1 < page_count,
                        |this, _, _| this.page += 1,
                    )),
            );
        }
        let page_entries = entries
            .into_iter()
            .skip(self.page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .collect::<Vec<_>>();
        self.inputs
            .retain(|index, _| page_entries.iter().any(|(id, _)| id == index));
        let form = h_form().label_width(px(180.0)).children(
            page_entries
                .into_iter()
                .map(|(index, name)| self.field_row(index, name, window, cx)),
        );
        view.child(
            GroupBox::new()
                .id("save-fields")
                .outline()
                .when(self.tab == Tab::City, |group| {
                    group.title(Label::new(locale.text("city_data")))
                })
                .child(form),
        )
    }

    fn city_summary(&self, cx: &mut Context<Self>) -> Div {
        let locale = self.locale;
        let doc = self.current.as_ref().unwrap();
        let (level_key, xp_key) = doc.game.progression();
        let level = doc
            .fields
            .iter()
            .find(|field| field.snapshot == self.snapshot && field.path == level_key)
            .and_then(|field| field.value.parse::<i64>().ok());
        let xp = doc
            .fields
            .iter()
            .find(|field| field.snapshot == self.snapshot && field.path == xp_key)
            .and_then(|field| field.value.parse::<i64>().ok());
        let mut panel = column();
        if doc.is_pc2_city() {
            panel = panel.child(row(
                locale.text(if doc.pc2_survival() {
                    "survival_city"
                } else if doc.pc2_sandbox() {
                    "sandbox"
                } else {
                    "regular_city"
                }),
                "",
            ));
        }
        if let (Some(level), Some(xp)) = (level, xp)
            && let Some(total) = doc.game.next_level_xp(level)
        {
            panel = panel.child(
                row(
                    &format!("{} {level}", locale.text("level")),
                    &locale
                        .text("xp_progress")
                        .replace("{xp}", &xp.to_string())
                        .replace("{total}", &total.to_string()),
                )
                .child(
                    ProgressBar::new("xp-progress")
                        .value(xp as f32 / total as f32 * 100.0)
                        .color(rgb(accent(doc.game)))
                        .accessibility_label(locale.text("level")),
                )
                .child(div().flex().child(action(
                    cx,
                    "reset-xp",
                    locale.text("reset_xp"),
                    false,
                    xp != 0,
                    move |this, _, _| {
                        if let Some(doc) = &mut this.current
                            && let Some((i, field)) =
                                doc.fields.iter_mut().enumerate().find(|(_, field)| {
                                    field.snapshot == this.snapshot && field.path == xp_key
                                })
                        {
                            field.value = "0".to_owned();
                            this.inputs.remove(&i);
                        }
                    },
                ))),
            );
        }
        panel
    }

    fn field_row(
        &mut self,
        index: usize,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> form::Field {
        let locale = self.locale;
        let document = self.current.as_ref().unwrap();
        let field = &document.fields[index];
        let locked = field.locked(document.game)
            || (document.pc2_survival()
                && save::PC2_SANDBOX_FLAGS.contains(&field.path.as_str())
                && field.value == "false");
        let path = field.path.clone();
        let value = field.value.clone();
        let kind = field.kind;
        let help = match path.as_str() {
            "money" | "stats.money" => locale.text("money_help"),
            "level" | "stats.level" => locale.text("level_help"),
            "unlockAll" => locale.text("unlock_all_help"),
            _ if field.is_game_record(document.game) => {
                let id = path.split_once('.').map_or("", |(_, id)| id);
                locale
                    .get(&format!("record_{id}_help"))
                    .unwrap_or_else(|| locale.text("game_records_help"))
            }
            _ => "",
        };
        let item = form::field()
            .label(name.clone())
            .items_center()
            .when(!help.is_empty(), |field| field.description(help));
        if kind == Kind::Boolean {
            return item.child(
                Checkbox::new(("checkbox", index))
                    .accessibility_label(name)
                    .checked(value == "true")
                    .disabled(locked)
                    .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                        if let Some(doc) = &mut this.current {
                            doc.fields[index].value = checked.to_string();
                        }
                        cx.notify();
                    })),
            );
        }
        if let Some(options) = field_options(&path, locale) {
            let selected = options
                .iter()
                .find(|entry| entry.0 == value)
                .map_or(value.as_str(), |entry| entry.1);
            let selected = selected.to_owned();
            let editor = cx.weak_entity();
            return item.child(
                div().flex().child(
                    Button::new(("field-picker", index))
                        .label(selected)
                        .icon(IconName::ChevronDown)
                        .outline()
                        .small()
                        .disabled(locked)
                        .dropdown_menu(move |menu, _, _| {
                            options.iter().fold(menu, |menu, &(key, label)| {
                                let editor = editor.clone();
                                menu.item(PopupMenuItem::new(label).checked(key == value).on_click(
                                    move |_, _, cx| {
                                        let _ = editor.update(cx, |this, cx| {
                                            if let Some(doc) = &mut this.current {
                                                doc.fields[index].value = key.to_owned();
                                            }
                                            cx.notify();
                                        });
                                    },
                                ))
                            })
                        }),
                ),
            );
        }
        if locked {
            return item.child(subtle(value));
        }
        let input = &self
            .inputs
            .entry(index)
            .or_insert_with(|| {
                let state = cx.new(|cx| InputState::new(window, cx).default_value(value));
                let subscription =
                    cx.subscribe(&state, move |this, state, event: &InputEvent, cx| {
                        if matches!(event, InputEvent::Change)
                            && let Some(doc) = &mut this.current
                        {
                            doc.fields[index].value = state.read(cx).value().to_string();
                            cx.notify();
                        }
                    });
                (state, subscription)
            })
            .0;
        item.child(Input::new(input).w_full().max_w(px(280.0)))
    }
}

impl Render for Editor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("save-editor")
            .size_full()
            .flex()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                if let Some(path) = paths.paths().first() {
                    this.open(path);
                    if this.notice.is_none() {
                        this.query
                            .update(cx, |state, cx| state.set_value("", window, cx));
                    }
                    cx.notify();
                }
            }))
            .child(self.sidebar(cx))
            .child(self.document(window, cx))
    }
}

fn library_action(id: &'static str, label: &'static str, icon: IconName, menu: bool) -> Button {
    Button::new(id)
        .ghost()
        .w_full()
        .accessibility_label(label)
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .gap_2()
                .child(icon)
                .child(div().min_w_0().flex_1().truncate().child(label))
                .when(menu, |row| row.child(IconName::ChevronDown)),
        )
}

fn action(
    cx: &Context<Editor>,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    enabled: bool,
    handler: impl Fn(&mut Editor, &mut Window, &mut Context<Editor>) + 'static,
) -> Button {
    let button = Button::new(id).label(label).small().disabled(!enabled);
    let button = if selected {
        button.primary()
    } else {
        button.outline()
    };
    button.on_click(cx.listener(move |this, _, window, cx| {
        handler(this, window, cx);
        cx.notify();
    }))
}

fn column() -> Div {
    div().flex().flex_col().gap_2().pb_3()
}
#[track_caller]
fn subtle(text: impl Into<SharedString>) -> Label {
    let text = text.into();
    let caller = std::panic::Location::caller();
    Label::new(text.clone())
        .id(SharedString::from(format!(
            "{}:{}:{}:{text}",
            caller.file(),
            caller.line(),
            caller.column()
        )))
        .text_sm()
        .text_color(rgb(MUTED))
}
fn row(title: &str, help: &str) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!("row-{title}")))
        .border_b_1()
        .border_color(rgb(BORDER))
        .py_3()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().font_semibold().child(Label::new(title.to_owned())))
        .when(!help.is_empty(), |row| row.child(subtle(help.to_owned())))
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
            | "randomDisasters"
            | "survivalLifelines"
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
            "globalFlags" | "flags" => {
                return locale.get(&format!("record_{id}")).map_or_else(
                    || format!("{} · {}", locale.text("record"), id),
                    str::to_owned,
                );
            }
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
