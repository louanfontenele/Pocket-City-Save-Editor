use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

use crate::{
    catalog::{Collection, MAX_AFFINITY},
    locale::Locale,
    save::{self, Game, Kind, Save},
};

mod progression;
use progression::ProgressEdit;

pub type EditorState = Mutex<Editor>;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Request {
    View,
    Refresh,
    OpenFile,
    OpenDrop,
    AddFolder,
    OpenEntry { path: PathBuf },
    Language { code: String },
    Snapshot { index: usize },
    Collection { key: String },
    Progress { edit: ProgressEdit },
    ResetXp,
    Save,
    Reload,
    Export,
    Restore { path: PathBuf },
}

struct Entry {
    path: PathBuf,
    title: String,
    game: Game,
}

pub struct Editor {
    library: Vec<Entry>,
    folders: Vec<PathBuf>,
    current: Option<Save>,
    snapshot: usize,
    locale: Locale,
    generation: u64,
    notice: Option<(bool, String)>,
    dropped: Option<PathBuf>,
}

impl Editor {
    pub fn new() -> Self {
        let locale = locale_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|code| Locale::from_code(code.trim()))
            .unwrap_or_default();
        let mut editor = Self {
            library: Vec::new(),
            folders: save::default_folders()
                .into_iter()
                .filter(|path| path.is_dir())
                .collect(),
            current: None,
            snapshot: 0,
            locale,
            generation: 0,
            notice: None,
            dropped: None,
        };
        if let Err(error) = editor.refresh() {
            editor.failure(error);
        }
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
        if let Some(path) = initial
            && let Err(error) = editor.open(&path)
        {
            editor.failure(error);
        }
        editor
    }

    fn failure(&mut self, error: anyhow::Error) {
        self.notice = Some((true, format!("{error:#}")));
    }

    fn note(&mut self, key: &str) {
        self.notice = Some((false, self.locale.text(key).to_owned()));
    }

    fn refresh(&mut self) -> Result<()> {
        self.library.retain(|entry| entry.path.is_file());
        let mut failure = None;
        for folder in self.folders.clone() {
            match save::scan(&folder) {
                Ok(paths) => {
                    for path in paths {
                        if let Ok(document) = Save::open(&path) {
                            self.remember(&document);
                        }
                    }
                }
                Err(error) => failure = Some(error),
            }
        }
        failure.map_or(Ok(()), Err)
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
        self.library
            .sort_by_cached_key(|entry| (entry.title.to_lowercase(), entry.path.clone()));
    }

    fn check_pending(&self) -> Result<()> {
        ensure!(
            !self
                .current
                .as_ref()
                .is_some_and(|save| save.changed() != 0),
            "{}",
            self.locale.text("pending_guard")
        );
        Ok(())
    }

    fn open(&mut self, path: &Path) -> Result<()> {
        self.check_pending()?;
        let document = Save::open(path)?;
        self.snapshot = document.latest;
        self.replace(document);
        Ok(())
    }

    pub fn queue_drop(&mut self, path: PathBuf) {
        self.dropped = Some(path);
    }

    fn replace(&mut self, document: Save) {
        self.snapshot = self.snapshot.min(document.snapshots.len() - 1);
        self.remember(&document);
        self.current = Some(document);
        self.notice = None;
        self.generation += 1;
    }

    fn dispatch(&mut self, request: Request, generation: Option<u64>) -> Result<()> {
        match request {
            Request::View => return Ok(()),
            Request::Refresh => {
                self.notice = None;
                return self.refresh();
            }
            Request::OpenFile => {
                self.check_pending()?;
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.open(&path)?;
                }
                return Ok(());
            }
            Request::AddFolder => {
                if let Some(path) = rfd::FileDialog::new().pick_folder()
                    && !self.folders.contains(&path)
                {
                    self.folders.push(path);
                    self.notice = None;
                    self.refresh()?;
                }
                return Ok(());
            }
            Request::OpenEntry { path } => {
                ensure!(
                    self.library.iter().any(|entry| entry.path == path),
                    "Unknown library entry"
                );
                return self.open(&path);
            }
            Request::OpenDrop => {
                if let Some(path) = self.dropped.take() {
                    self.open(&path)?;
                }
                return Ok(());
            }
            Request::Language { code } => {
                let locale = Locale::from_code(&code);
                ensure!(locale.code() == code, "Unknown language");
                self.locale = locale;
                self.notice = None;
                if let Some(path) = locale_path()
                    && let Some(folder) = path.parent()
                {
                    std::fs::create_dir_all(folder)?;
                    std::fs::write(path, locale.code())?;
                }
                return Ok(());
            }
            _ => {}
        }
        ensure!(
            generation == Some(self.generation),
            "The open save has changed"
        );
        let locale = self.locale;
        let snapshot = self.snapshot;
        let document = self.current.as_mut().context("Open a save first")?;
        self.notice = None;
        match request {
            Request::Snapshot { index } => {
                ensure!(index < document.snapshots.len(), "Unknown city version");
                if snapshot != index {
                    self.snapshot = index;
                    self.generation += 1;
                }
            }
            Request::Collection { key } => {
                let (collection, value) = match key.as_str() {
                    "relationships" => (Collection::Affinity, MAX_AFFINITY.to_string()),
                    "forceUnlockedBuildings" => (Collection::Buildings, "true".to_owned()),
                    "unlockedCars" => (Collection::Vehicles, "true".to_owned()),
                    _ => anyhow::bail!("Unknown collection action"),
                };
                for field in &mut document.fields {
                    if field.snapshot == snapshot
                        && collection.matches(&field.path)
                        && !field.locked(document.game)
                    {
                        field.value.clone_from(&value);
                    }
                }
            }
            Request::Progress { edit } => {
                let progress = document.progress[snapshot]
                    .as_mut()
                    .context("No city progression in this save")?;
                edit.apply(progress)?;
            }
            Request::ResetXp => {
                let (_, xp) = document.game.progression();
                let field = document
                    .fields
                    .iter_mut()
                    .find(|field| field.snapshot == snapshot && field.path == xp)
                    .context("No city experience in this save")?;
                field.value = "0".to_owned();
            }
            Request::Save => {
                if document.changed() != 0 {
                    let saved = document.save()?;
                    self.replace(saved);
                    self.note("saved");
                }
            }
            Request::Reload => {
                let saved = Save::open(&document.path)?;
                self.replace(saved);
                self.note("reloaded");
            }
            Request::Export => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(save::file_label(&document.path))
                    .save_file()
                {
                    document.export(&path)?;
                    self.note("exported");
                }
            }
            Request::Restore { path } => {
                ensure!(document.changed() == 0, "{}", locale.text("pending_guard"));
                ensure!(document.backups()?.contains(&path), "Unknown backup");
                let restored = document.restore(&path)?;
                self.replace(restored);
                self.note("restored");
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn apply_drafts(&mut self, drafts: Vec<Draft>, generation: Option<u64>) -> Result<()> {
        if drafts.is_empty() {
            return Ok(());
        }
        ensure!(
            generation == Some(self.generation),
            "The open save has changed"
        );
        let document = self.current.as_mut().context("Open a save first")?;
        for draft in &drafts {
            let field = document
                .fields
                .get(draft.index)
                .context("Unknown save field")?;
            ensure!(
                field.snapshot == self.snapshot,
                "Field belongs to another city version"
            );
            ensure!(
                !field_locked(document, draft.index),
                "{}",
                self.locale.text("protected")
            );
        }
        for draft in drafts {
            document.fields[draft.index].value = draft.value;
        }
        Ok(())
    }

    fn view(&self) -> Value {
        let locale = self.locale;
        let library = self
            .library
            .iter()
            .map(|entry| {
                json!({
                    "path": entry.path,
                    "title": settings_title(&entry.path, locale).unwrap_or(&entry.title),
                    "game": game_number(entry.game),
                    "kind": archive_key(&entry.path),
                    "filename": save::file_label(&entry.path),
                })
            })
            .collect::<Vec<_>>();
        json!({
            "locale": locale.code(),
            "languageName": locale.name(),
            "languages": Locale::ALL,
            "library": library,
            "generation": self.generation,
            "document": self.current.as_ref().map(|document| self.document_view(document)),
            "notice": self.notice.as_ref().map(|(error, message)| json!({ "error": error, "message": message })),
        })
    }

    fn document_view(&self, document: &Save) -> Value {
        let locale = self.locale;
        let snapshot = self.snapshot;
        let (level_key, xp_key) = document.game.progression();
        let mut level = None;
        let mut xp = None;
        let fields = document.fields.iter().enumerate().filter_map(|(index, field)| {
            if field.snapshot != snapshot {
                return None;
            }
            if field.path == level_key {
                level = field.value.parse::<i64>().ok();
            } else if field.path == xp_key {
                xp = field.value.parse::<i64>().ok();
            }
            let help = match field.path.as_str() {
                "money" | "stats.money" => locale.text("money_help"),
                "level" | "stats.level" => locale.text("level_help"),
                "unlockAll" => locale.text("unlock_all_help"),
                _ if field.is_game_record(document.game) => {
                    let id = field.path.split_once('.').map_or("", |(_, id)| id);
                    locale.get(&format!("record_{id}_help")).unwrap_or_else(|| locale.text("game_records_help"))
                }
                _ => "",
            };
            Some(json!({
                "index": index,
                "path": field.path,
                "name": field_name(&field.path, &field.label, locale),
                "value": field.value,
                "type": match field.kind { Kind::Boolean => "boolean", Kind::Integer | Kind::Decimal => "number", _ => "text" },
                "locked": field_locked(document, index),
                "changed": field.changed(),
                "overview": field_section(&field.path) == Tab::City,
                "collection": Collection::ALL.into_iter().find(|choice| choice.matches(&field.path)).map(Collection::key),
                "help": help,
                "options": field_options(&field.path, locale),
            }))
        }).collect::<Vec<_>>();
        let snapshots = document
            .snapshots
            .iter()
            .enumerate()
            .map(|(index, label)| {
                let (kind, number) = label.split_once(' ').unwrap_or(("", ""));
                if document.snapshots.len() == 1 {
                    return label.clone();
                }
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
            })
            .collect::<Vec<_>>();
        let collections = Collection::ALL
            .into_iter()
            .filter(|choice| {
                document
                    .fields
                    .iter()
                    .any(|field| field.snapshot == snapshot && choice.matches(&field.path))
            })
            .map(|choice| {
                json!({
                    "key": choice.key(),
                    "label": locale.text(choice.label_key()),
                    "help": match choice {
                        Collection::Affinity => locale.text("affinity_help"),
                        Collection::Buildings => locale.text("building_help"),
                        Collection::GlobalFlags => locale.text("game_records_help"),
                        _ if choice.is_resource_automation() => locale.text("auto_help"),
                        _ => "",
                    },
                    "action": match choice {
                        Collection::Affinity => Some(locale.text("max_affinity")),
                        Collection::Buildings => Some(locale.text("unlock_buildings")),
                        Collection::Vehicles => Some(locale.text("unlock_cars")),
                        _ => None,
                    },
                })
            })
            .collect::<Vec<_>>();
        let (backups, backup_error) = match document.backups() {
            Ok(paths) => (
                paths
                    .into_iter()
                    .map(|path| {
                        let label = if path.extension().is_some_and(|ext| ext == "bk") {
                            locale.text("game_backup").to_owned()
                        } else {
                            save::file_label(&path)
                        };
                        json!({ "path": path, "label": label })
                    })
                    .collect::<Vec<_>>(),
                None,
            ),
            Err(error) => (Vec::new(), Some(error.to_string())),
        };
        json!({
            "path": document.path,
            "filename": save::file_label(&document.path),
            "title": settings_title(&document.path, locale).unwrap_or(&document.title),
            "game": game_number(document.game),
            "gameTitle": document.game.title(),
            "kind": archive_key(&document.path),
            "mode": document.is_pc2_city().then(|| if document.pc2_survival() { "survival_city" } else if document.pc2_sandbox() { "sandbox" } else { "regular_city" }),
            "archiveNote": matches!(archive_key(&document.path), "auto_archive" | "manual_archive") && !document.pc2_survival(),
            "pending": document.changed(),
            "snapshot": snapshot,
            "snapshots": snapshots,
            "fields": fields,
            "collections": collections,
            "level": level,
            "xp": xp,
            "nextXp": level.and_then(|level| document.game.next_level_xp(level)),
            "progression": document.progress[snapshot].as_ref().map(|progress| progression::view(progress, locale)),
            "backups": backups,
            "backupError": backup_error,
        })
    }
}

#[derive(Deserialize)]
pub struct Draft {
    index: usize,
    value: String,
}

#[tauri::command]
pub async fn editor_request(
    app: AppHandle,
    request: Request,
    generation: Option<u64>,
    drafts: Vec<Draft>,
) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<EditorState>();
        let mut editor = state.lock().map_err(|error| error.to_string())?;
        if let Err(error) = editor
            .apply_drafts(drafts, generation)
            .and_then(|()| editor.dispatch(request, generation))
        {
            editor.failure(error);
        }
        Ok(editor.view())
    })
    .await
    .map_err(|error| error.to_string())?
}

fn field_locked(document: &Save, index: usize) -> bool {
    let field = &document.fields[index];
    field.locked(document.game)
        || (document.pc2_survival()
            && save::PC2_SANDBOX_FLAGS.contains(&field.path.as_str())
            && field.value == "false")
}

fn game_number(game: Game) -> u8 {
    match game {
        Game::One => 1,
        Game::Two => 2,
    }
}

fn locale_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("Pocket City Save Editor").join("language"))
}

#[derive(PartialEq, Eq)]
enum Tab {
    City,
    Advanced,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_drafts_guard_the_open_save_and_its_snapshot() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("_city_draft.save");
        let original = r#"{"AS":[{"timestamp":100,"metadata":{"name":"Draft"},"stats":{"money":42}}],"MS":[{"timestamp":200,"metadata":{"name":"Draft"},"stats":{"money":51}}]}"#;
        std::fs::write(&path, original)?;
        let mut editor = Editor {
            library: Vec::new(),
            folders: Vec::new(),
            current: None,
            snapshot: 0,
            locale: Locale::default(),
            generation: 0,
            notice: None,
            dropped: None,
        };
        editor.open(&path)?;
        let initial = editor.generation;
        let document = editor.current.as_ref().unwrap();
        let balance = document
            .fields
            .iter()
            .position(|field| field.path == "stats.money" && field.snapshot == editor.snapshot)
            .context("Missing current balance")?;
        let protected = document
            .fields
            .iter()
            .position(|field| field.path == "metadata.name" && field.snapshot == editor.snapshot)
            .context("Missing protected title")?;
        assert!(
            editor
                .apply_drafts(
                    vec![
                        Draft {
                            index: balance,
                            value: "99".to_owned()
                        },
                        Draft {
                            index: protected,
                            value: "Oversight".to_owned()
                        },
                    ],
                    Some(initial)
                )
                .is_err()
        );
        assert_eq!(editor.current.as_ref().unwrap().changed(), 0);
        editor.apply_drafts(
            vec![Draft {
                index: balance,
                value: "99".to_owned(),
            }],
            Some(initial),
        )?;
        assert!(editor.open(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path)?, original);
        editor.dispatch(Request::Snapshot { index: 0 }, Some(initial))?;
        assert!(
            editor
                .apply_drafts(
                    vec![Draft {
                        index: balance,
                        value: "77".to_owned()
                    }],
                    Some(initial)
                )
                .is_err()
        );
        editor.dispatch(Request::Save, Some(editor.generation))?;
        assert_eq!(
            std::fs::read_to_string(&path)?,
            original.replace("\"money\":51", "\"money\":99")
        );
        Ok(())
    }
}
