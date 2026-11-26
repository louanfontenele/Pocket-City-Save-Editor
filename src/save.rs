use crate::catalog::{Collection, MAX_AFFINITY, survival_max};
use anyhow::{Context, Result, ensure};
use flate2::{Compression, GzBuilder, bufread::GzDecoder};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    ops::Range,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

#[path = "progress.rs"]
pub mod progress;
use progress::Progress;

const MAX_BYTES: usize = 64 * 1024 * 1024;
pub const PC2_SANDBOX_FLAGS: [&str; 4] =
    ["unlockAll", "infiniteMoney", "disableQuests", "maxLevel"];
const EMPTY_TILE: &str = "\"\",";
const PC1_IMPLICIT_KEYS: [(&str, &str); 4] = [
    ("_x_", "fields"),
    ("_y_", "class"),
    ("_z_", "active"),
    ("_2_", "extra"),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Game {
    One,
    Two,
}

impl Game {
    pub fn title(self) -> &'static str {
        match self {
            Self::One => "Pocket City",
            Self::Two => "Pocket City 2",
        }
    }

    pub fn progression(self) -> (&'static str, &'static str) {
        match self {
            Self::One => ("stats.level", "stats.exp"),
            Self::Two => ("level", "XP"),
        }
    }

    pub fn next_level_xp(self, level: i64) -> Option<i64> {
        let (base, step) = match self {
            Self::One => (1500, 150),
            Self::Two => (2000, 40),
        };
        let threshold = level.checked_sub(1)?.checked_mul(step)?.checked_add(base)?;
        (level > 0 && (self == Self::One || threshold <= i64::from(i32::MAX))).then_some(threshold)
    }

    fn protected(self, path: &str) -> bool {
        let key = path.rsplit('.').next().unwrap_or(path);
        matches!(
            key,
            "FILE_ID"
                | "__type"
                | "_ES3Ref"
                | "parentCity"
                | "regionIndexI"
                | "regionIndexJ"
                | "buildVersion"
                | "mapSize"
                | "isSurvivalMode"
                | "isTutorialPreset"
                | "SEED"
                | "rng"
        ) || path.starts_with("regionFileIDs.")
            || (self == Self::Two
                && [
                    "researched.",
                    "disabledResearch.",
                    "pets.",
                    "petStates.",
                    "completedQuests.",
                    "eventCooldowns.",
                ]
                .iter()
                .any(|prefix| path.starts_with(prefix)))
            || (self == Self::One
                && (path.starts_with("metadata.")
                    || path.starts_with("stats.globalSettings.chestunlockbuildings.")
                    || matches!(key, "size" | "index" | "uid" | "timestamp")))
    }
}

#[derive(Debug)]
struct Node {
    span: Range<usize>,
    key_start: usize,
    children: Vec<(String, Node)>,
}

impl Node {
    fn get(&self, key: &str) -> Option<&Self> {
        self.children
            .iter()
            .find_map(|(name, value)| (name == key).then_some(value))
    }
}

struct Parser<'a> {
    source: &'a str,
    position: usize,
}

impl Parser<'_> {
    fn next(&mut self) -> Option<u8> {
        while matches!(
            self.source.as_bytes().get(self.position),
            Some(b' ' | b'\n' | b'\r' | b'\t')
        ) {
            self.position += 1;
        }
        self.source.as_bytes().get(self.position).copied()
    }

    fn consume(&mut self, byte: u8) -> Result<()> {
        ensure!(
            self.next() == Some(byte),
            "Expected '{}' at byte {}",
            byte as char,
            self.position
        );
        self.position += 1;
        Ok(())
    }

    fn value(&mut self, depth: usize) -> Result<Node> {
        ensure!(depth <= 128, "Save nesting exceeds 128 levels");
        let token = self.next().context("Unexpected end of save")?;
        let start = self.position;
        let mut children = Vec::new();
        match token {
            b'{' | b'[' => {
                self.position += 1;
                let close = if token == b'{' { b'}' } else { b']' };
                if self.next() != Some(close) {
                    loop {
                        self.next();
                        let key_start = self.position;
                        let key = if token == b'{'
                            && let Some((marker, name)) =
                                PC1_IMPLICIT_KEYS.iter().find(|(marker, _)| {
                                    self.source[self.position..].starts_with(marker)
                                }) {
                            self.position += marker.len();
                            (*name).to_owned()
                        } else if token == b'{' {
                            let key = self.value(depth + 1)?;
                            let raw = &self.source[key.span];
                            let key = serde_json::from_str::<String>(raw)
                                .unwrap_or_else(|_| raw.to_owned());
                            self.consume(b':')?;
                            key
                        } else {
                            children.len().to_string()
                        };
                        let mut value = self.value(depth + 1)?;
                        value.key_start = key_start;
                        children.push((key, value));
                        if self.next() != Some(b',') {
                            break;
                        }
                        self.position += 1;
                        if self.next() == Some(close) {
                            break;
                        }
                    }
                }
                self.consume(close)?;
            }
            b'"' => {
                self.position += 1;
                loop {
                    let byte = *self
                        .source
                        .as_bytes()
                        .get(self.position)
                        .context("Unterminated string")?;
                    self.position += 1;
                    match byte {
                        b'\\' => self.position += 1,
                        b'"' => break,
                        _ => {}
                    }
                }
            }
            _ => {
                while let Some(byte) = self.source.as_bytes().get(self.position) {
                    if matches!(
                        byte,
                        b' ' | b'\n' | b'\r' | b'\t' | b',' | b':' | b']' | b'}'
                    ) {
                        break;
                    }
                    self.position += 1;
                }
                ensure!(self.position > start, "Invalid token at byte {start}");
            }
        }
        Ok(Node {
            span: start..self.position,
            key_start: start,
            children,
        })
    }
}

enum Encoding {
    Plain,
    Gzip(flate2::GzHeader),
    Tiles,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Integer,
    Decimal,
    Boolean,
}

pub struct Field {
    pub path: String,
    pub label: String,
    pub value: String,
    initial: String,
    range: Range<usize>,
    pub kind: Kind,
    pub snapshot: usize,
    addition: Option<Addition>,
}

#[derive(Clone, Copy)]
struct Addition {
    key: i32,
    requires_comma: bool,
    wrapper: Option<&'static str>,
}

#[derive(Default)]
struct MemberInsertions {
    requires_comma: bool,
    groups: BTreeMap<Option<&'static str>, Vec<String>>,
}

impl Field {
    pub fn changed(&self) -> bool {
        self.initial != self.value
    }

    pub fn is_game_record(&self, game: Game) -> bool {
        game == Game::Two
            && (Collection::GlobalFlags.matches(&self.path) || self.path.starts_with("flags."))
    }

    pub fn locked(&self, game: Game) -> bool {
        game.protected(&self.path)
            || self.is_game_record(game)
            || (game == Game::Two
                && Collection::Buildings.matches(&self.path)
                && self.addition.is_none())
    }

    fn encode(&self, game: Game) -> Result<String> {
        ensure!(!self.locked(game), "Campo protegido");
        let text = self.value.trim();
        match self.kind {
            Kind::Text => {
                if game == Game::One {
                    ensure!(
                        !self.value.contains('^'),
                        "{} cannot contain ^ in Pocket City saves",
                        self.label
                    );
                    ensure!(
                        !self.value.as_bytes().windows(3).any(|bytes| {
                            bytes[0] == b'_'
                                && bytes[2] == b'_'
                                && (bytes[1].is_ascii_lowercase() || bytes[1].is_ascii_digit())
                        }),
                        "{} contains a reserved Pocket City marker",
                        self.label
                    );
                }
                Ok(serde_json::to_string(&self.value)?)
            }
            Kind::Integer => {
                let value = text.parse::<i64>().context("Enter a whole number")?;
                if Collection::Affinity.matches(&self.path) {
                    ensure!(
                        (0..=MAX_AFFINITY).contains(&value),
                        "A afinidade deve estar entre 0 e {MAX_AFFINITY}"
                    );
                }
                if let Some(id) = self
                    .path
                    .strip_prefix("upgradesSpent.")
                    .or_else(|| self.path.strip_prefix("upgradesSpentCity."))
                    .and_then(|key| key.parse().ok())
                    && let Some(maximum) = survival_max(id)
                {
                    ensure!(
                        (i64::from(Collection::CitySurvival.matches(&self.path))..=maximum)
                            .contains(&value),
                        "Invalid survival upgrade level (maximum {maximum})"
                    );
                }
                if [Collection::AutoSell, Collection::AutoConvert]
                    .iter()
                    .any(|collection| collection.matches(&self.path))
                {
                    ensure!(
                        value >= -1,
                        "Enter −1 to disable automation or a nonnegative stock amount"
                    );
                }
                if game == Game::One {
                    ensure!(
                        value.unsigned_abs() <= 9_007_199_254_740_991,
                        "Outside the range of exact Pocket City integers"
                    );
                } else if self.path != "money"
                    && (self.addition.is_some() || self.initial.parse::<i32>().is_ok())
                {
                    ensure!(
                        i32::try_from(value).is_ok(),
                        "Outside the range of a 32-bit field"
                    );
                }
                Ok(value.to_string())
            }
            Kind::Decimal => {
                let value = text
                    .parse::<f64>()
                    .context("Enter a number using a decimal point")?;
                ensure!(value.is_finite(), "Enter a finite number");
                Ok(value.to_string())
            }
            Kind::Boolean => Ok(text.parse::<bool>()?.to_string()),
        }
    }
}

pub struct Save {
    pub game: Game,
    pub path: PathBuf,
    pub title: String,
    pub fields: Vec<Field>,
    pub snapshots: Vec<String>,
    pub latest: usize,
    pub progress: Vec<Option<Progress>>,
    source: String,
    encoding: Encoding,
    original: Vec<u8>,
}

impl Save {
    pub fn is_pc2_city(&self) -> bool {
        self.game == Game::Two && self.fields.iter().any(|field| field.path == "FILE_ID")
    }

    pub fn pc2_sandbox(&self) -> bool {
        self.fields
            .iter()
            .any(|field| PC2_SANDBOX_FLAGS.contains(&field.path.as_str()) && field.value == "true")
    }

    pub fn pc2_survival(&self) -> bool {
        self.fields
            .iter()
            .any(|field| field.path == "isSurvivalMode" && field.value == "true")
    }

    pub fn open(path: &Path) -> Result<Self> {
        Self::decode(path.to_owned(), read_limited(path)?)
    }

    fn decode(path: PathBuf, original: Vec<u8>) -> Result<Self> {
        let (source, mut encoding) = if original.starts_with(&[0x1f, 0x8b]) {
            let mut gzip = GzDecoder::new(original.as_slice());
            let header = gzip.header().context("Invalid gzip header")?.clone();
            let mut data = Vec::new();
            (&mut gzip)
                .take(MAX_BYTES as u64 + 1)
                .read_to_end(&mut data)?;
            ensure!(data.len() <= MAX_BYTES, "Uncompressed save exceeds 64 MiB");
            ensure!(
                gzip.into_inner().is_empty(),
                "Unexpected data after gzip member"
            );
            (String::from_utf8(data)?, Encoding::Gzip(header))
        } else {
            (std::str::from_utf8(&original)?.to_owned(), Encoding::Plain)
        };
        let source = if matches!(encoding, Encoding::Plain)
            && ((file_game(&path) == Some(Game::One) && file_label(&path) != "_user.save")
                || source
                    .trim_start_matches(['\u{feff}', ' ', '\r', '\n', '\t'])
                    .starts_with("{\"AS\""))
        {
            encoding = Encoding::Tiles;
            expand_tiles(&source)?
        } else {
            source
        };
        let mut parser = Parser {
            source: &source,
            position: usize::from(source.starts_with('\u{feff}')) * 3,
        };
        let root = parser.value(0)?;
        ensure!(parser.next().is_none(), "Unexpected data after save");
        let (game, records, latest) =
            if let Some(city) = root.get("CITY").and_then(|key| key.get("value")) {
                ensure!(
                    city.get("FILE_ID").is_some() && city.get("money").is_some(),
                    "Unexpected CITY record"
                );
                (Game::Two, vec![(String::new(), city)], 0)
            } else if let Some(settings) = root
                .get("SETTINGS")
                .or_else(|| root.get("OS"))
                .and_then(|key| key.get("value"))
            {
                (Game::Two, vec![(String::new(), settings)], 0)
            } else if file_label(&path) == "_user.save" && source.trim_start().starts_with('{') {
                (Game::One, vec![(String::new(), &root)], 0)
            } else {
                let automatic = root
                    .get("AS")
                    .context("Missing Pocket City autosave list")?;
                let manual = root
                    .get("MS")
                    .context("Missing Pocket City manual save list")?;
                let timestamp = |node: Option<&Node>| {
                    node.and_then(|node| node.get("timestamp"))
                        .and_then(|node| source[node.span.clone()].parse::<u64>().ok())
                };
                let latest = if timestamp(manual.get("0")) > timestamp(automatic.get("0")) {
                    automatic.children.len()
                } else {
                    0
                };
                let mut records = Vec::new();
                for (list, label) in [(automatic, "Automático"), (manual, "Manual")] {
                    for (index, (_, snapshot)) in list.children.iter().enumerate() {
                        ensure!(
                            snapshot.get("metadata").is_some() && snapshot.get("stats").is_some(),
                            "Invalid Pocket City snapshot"
                        );
                        records.push((format!("{label} {}", index + 1), snapshot));
                    }
                }
                ensure!(
                    !records.is_empty(),
                    "Pocket City save contains no snapshots"
                );
                (Game::One, records, latest)
            };
        let mut fields = Vec::new();
        let mut snapshots = Vec::new();
        let mut progress = Vec::new();
        for (index, (label, record)) in records.into_iter().enumerate() {
            visit_fields(record, &source, "", index, &mut fields);
            if game == Game::Two {
                add_catalog_fields(record, &source, &mut fields);
            }
            snapshots.push(label);
            progress.push(Progress::read(game, record, &source));
        }
        fields.sort_by_key(|field| field.range.start);
        let title = ["name", "cityName", "metadata.name", "metadata.cityName"]
            .iter()
            .find_map(|path| {
                fields
                    .iter()
                    .find(|f| f.path == *path && f.kind == Kind::Text && f.snapshot == latest)
            })
            .map(|f| f.value.clone())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| file_label(&path));
        Ok(Self {
            game,
            path,
            title,
            fields,
            snapshots,
            latest,
            progress,
            source,
            encoding,
            original,
        })
    }

    pub fn changed(&self) -> usize {
        self.fields.iter().filter(|field| field.changed()).count()
            + self
                .progress
                .iter()
                .flatten()
                .map(Progress::changed)
                .sum::<usize>()
    }

    fn output(&self) -> Result<Vec<u8>> {
        if self.changed() == 0 {
            return Ok(self.original.clone());
        }
        self.validate_progression()?;
        if self.is_pc2_city() && self.pc2_survival() {
            ensure!(
                !self.fields.iter().any(|field| {
                    PC2_SANDBOX_FLAGS.contains(&field.path.as_str())
                        && field.changed()
                        && field.value == "true"
                }),
                "Sandbox flags cannot be enabled in a survival city"
            );
        }
        let mut text = self.source.clone();
        let mut replacements = Vec::new();
        let mut insertions = BTreeMap::<(usize, usize), MemberInsertions>::new();
        for field in self.fields.iter().filter(|field| field.changed()) {
            let replacement = field
                .encode(self.game)
                .with_context(|| field.path.clone())?;
            if let Some(addition) = field.addition {
                let insertion = insertions
                    .entry((field.range.start, field.range.end))
                    .or_insert_with(|| MemberInsertions {
                        requires_comma: addition.requires_comma,
                        ..Default::default()
                    });
                insertion
                    .groups
                    .entry(addition.wrapper)
                    .or_default()
                    .push(format!("{}:{replacement}", addition.key));
            } else {
                replacements.push((field.range.clone(), replacement));
            }
        }
        for ((start, end), insertion) in insertions {
            let prefix = if insertion.requires_comma { "," } else { "" };
            let mut groups = Vec::new();
            for (wrapper, members) in insertion.groups {
                let members = members.join(",");
                groups.push(match wrapper {
                    Some(key) => format!("\"{key}\":{{{members}}}"),
                    None => members,
                });
            }
            let members = groups.join(",");
            let replacement = if start != end {
                format!("{{{members}}}")
            } else {
                format!("{prefix}{members}")
            };
            replacements.push((start..end, replacement));
        }
        replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
        for (range, replacement) in replacements {
            text.replace_range(range, &replacement);
        }
        if self
            .progress
            .iter()
            .flatten()
            .any(|progress| progress.changed() > 0)
        {
            text = progress::apply(&text, self.game, &self.progress)?;
        }
        let encoded = match &self.encoding {
            Encoding::Plain => text.into_bytes(),
            Encoding::Tiles => pack_tiles(&text).into_bytes(),
            Encoding::Gzip(header) => {
                let mut builder = GzBuilder::new()
                    .mtime(header.mtime())
                    .operating_system(header.operating_system());
                if let Some(filename) = header.filename() {
                    builder = builder.filename(filename);
                }
                if let Some(comment) = header.comment() {
                    builder = builder.comment(comment);
                }
                if let Some(extra) = header.extra() {
                    builder = builder.extra(extra);
                }
                let mut writer = builder.write(Vec::new(), Compression::default());
                writer.write_all(text.as_bytes())?;
                writer.finish()?
            }
        };
        let verified =
            Self::decode(self.path.clone(), encoded.clone()).context("Check rewritten save")?;
        ensure!(self.game == verified.game, "Save format changed");
        Ok(encoded)
    }

    fn validate_progression(&self) -> Result<()> {
        let (level_path, experience_path) = self.game.progression();
        for index in 0..self.snapshots.len() {
            let lookup = |path: &str| {
                self.fields
                    .iter()
                    .find(|field| field.snapshot == index && field.path == path)
            };
            let (Some(level), experience) = (lookup(level_path), lookup(experience_path)) else {
                continue;
            };
            if !level.changed() && !experience.is_some_and(Field::changed) {
                continue;
            }
            let level = level
                .value
                .trim()
                .parse::<i64>()
                .context("Digite um nível inteiro")?;
            let threshold = self
                .game
                .next_level_xp(level)
                .context("Nível fora do intervalo do jogo")?;
            if let Some(experience) = experience {
                let xp = experience
                    .value
                    .trim()
                    .parse::<i64>()
                    .context("Digite um valor inteiro de XP")?;
                ensure!(
                    (0..threshold).contains(&xp),
                    "O nível {level} aceita de 0 a {} XP. Ajuste o XP ou use Zerar XP.",
                    threshold - 1
                );
            }
        }
        Ok(())
    }

    pub fn save(&self) -> Result<Self> {
        let output = self.output()?;
        self.replace(&output)
    }

    pub fn export(&self, path: &Path) -> Result<()> {
        let output = self.output()?;
        let mut temporary = stage(path, &output)?;
        temporary.as_file_mut().sync_all()?;
        temporary
            .persist_noclobber(path)
            .context("Choose a new filename for the copy")?;
        Ok(())
    }

    pub fn backups(&self) -> Result<Vec<PathBuf>> {
        let folder = self.backup_dir()?;
        let mut paths = Vec::new();
        if folder.exists() {
            paths.extend(
                fs::read_dir(folder)?
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| path.extension().is_some_and(|ext| ext == "bak")),
            );
        }
        if self.game == Game::One {
            let path = self.game_backup();
            if path.is_file() {
                paths.push(path);
            }
        }
        paths.sort_by_key(|path| {
            std::cmp::Reverse(fs::metadata(path).and_then(|meta| meta.modified()).ok())
        });
        Ok(paths)
    }

    pub fn restore(&self, path: &Path) -> Result<Self> {
        ensure!(
            path.parent() == Some(self.backup_dir()?.as_path())
                || (self.game == Game::One && path == self.game_backup()),
            "Backup is from another file"
        );
        self.replace(&read_limited(path)?)
    }

    fn backup_dir(&self) -> Result<PathBuf> {
        let parent = parent(&self.path);
        let filename = self.path.file_name().context("Missing save filename")?;
        Ok(parent.join(".pocket-city-editor-backups").join(filename))
    }

    fn game_backup(&self) -> PathBuf {
        let mut filename = self.path.as_os_str().to_owned();
        filename.push(".bk");
        PathBuf::from(filename)
    }

    fn replace(&self, bytes: &[u8]) -> Result<Self> {
        let replacement = Self::decode(self.path.clone(), bytes.to_vec())?;
        ensure!(
            read_limited(&self.path)? == self.original,
            "The file changed on disk. Reload it before saving."
        );
        if bytes == self.original {
            return Ok(replacement);
        }
        let mut temporary = stage(&self.path, bytes)?;
        temporary.as_file_mut().sync_all()?;
        let folder = self.backup_dir()?;
        fs::create_dir_all(&folder)?;
        let mut backup = tempfile::Builder::new()
            .prefix("save-")
            .suffix(".bak")
            .tempfile_in(folder)?;
        backup.write_all(&self.original)?;
        backup.as_file_mut().sync_all()?;
        backup.keep()?;
        ensure!(
            read_limited(&self.path)? == self.original,
            "The file changed on disk. Reload it before saving."
        );
        temporary
            .persist(&self.path)
            .context("Replace the save file")?;
        Ok(replacement)
    }
}

fn stage(path: &Path, bytes: &[u8]) -> Result<NamedTempFile> {
    let mut temporary = NamedTempFile::new_in(parent(path))?;
    temporary.write_all(bytes)?;
    Ok(temporary)
}

fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn read_limited(path: &Path) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)
        .with_context(|| format!("Open {}", path.display()))?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut data)?;
    ensure!(data.len() <= MAX_BYTES, "Save exceeds 64 MiB");
    Ok(data)
}

fn visit_fields(node: &Node, source: &str, prefix: &str, snapshot: usize, fields: &mut Vec<Field>) {
    for (key, child) in &node.children {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        let raw = &source[child.span.clone()];
        if raw.starts_with('[') {
            continue;
        }
        if raw.starts_with('{') {
            visit_fields(child, source, &path, snapshot, fields);
            continue;
        }
        let kind = if raw.starts_with('"') {
            Kind::Text
        } else if matches!(raw, "true" | "false") {
            Kind::Boolean
        } else if raw.parse::<i64>().is_ok() {
            Kind::Integer
        } else if raw.parse::<f64>().is_ok_and(f64::is_finite) {
            Kind::Decimal
        } else {
            continue;
        };
        let value = if kind == Kind::Text {
            let Ok(value) = serde_json::from_str::<String>(raw) else {
                continue;
            };
            value
        } else {
            raw.to_owned()
        };
        fields.push(Field {
            path,
            label: humanize(key),
            initial: value.clone(),
            value,
            range: child.span.clone(),
            kind,
            snapshot,
            addition: None,
        });
    }
}

fn add_catalog_fields(root: &Node, source: &str, fields: &mut Vec<Field>) {
    for collection in Collection::ALL {
        let node = root.get(collection.key());
        let can_create = (matches!(collection, Collection::Affinity | Collection::Buildings)
            || collection.is_resource_automation())
            && root.get("FILE_ID").is_some();
        let (range, requires_comma, wrapper) = match node {
            Some(node) if source[node.span.clone()].starts_with('{') => {
                let (range, requires_comma) = object_end(node, source);
                (range, requires_comma, None)
            }
            Some(node) if can_create && &source[node.span.clone()] == "null" => {
                (node.span.clone(), false, None)
            }
            None if can_create => {
                let (range, requires_comma) = object_end(root, source);
                (range, requires_comma, Some(collection.key()))
            }
            _ => continue,
        };
        let kind = if matches!(collection, Collection::Vehicles | Collection::Buildings) {
            Kind::Boolean
        } else {
            Kind::Integer
        };
        for &(id, label) in collection.catalog() {
            let key = id.to_string();
            let path = format!("{}.{key}", collection.key());
            if let Some(field) = fields.iter_mut().find(|field| field.path == path) {
                field.label = label.to_owned();
                if collection == Collection::Buildings && field.kind == Kind::Boolean {
                    field.value = "true".to_owned();
                    field.initial.clone_from(&field.value);
                }
            } else if collection != Collection::CitySurvival
                && node.and_then(|node| node.get(&key)).is_none()
            {
                let value = if collection == Collection::Affinity {
                    "1"
                } else if collection.is_resource_automation() {
                    "-1"
                } else if kind == Kind::Boolean {
                    "false"
                } else {
                    "0"
                }
                .to_owned();
                fields.push(Field {
                    path,
                    label: label.to_owned(),
                    initial: value.clone(),
                    value,
                    range: range.clone(),
                    kind,
                    snapshot: 0,
                    addition: Some(Addition {
                        key: id,
                        requires_comma,
                        wrapper,
                    }),
                });
            }
        }
    }
}

fn object_end(node: &Node, source: &str) -> (Range<usize>, bool) {
    let offset = node.span.end - 1;
    let requires_comma =
        !node.children.is_empty() && !source[node.span.start..offset].trim_end().ends_with(',');
    (offset..offset, requires_comma)
}

fn humanize(key: &str) -> String {
    let mut result = String::new();
    let mut preceding_lowercase = false;
    for letter in key.chars() {
        if letter == '_' {
            result.push(' ');
        } else {
            if preceding_lowercase && letter.is_uppercase() {
                result.push(' ');
            }
            if result.is_empty() {
                result.extend(letter.to_uppercase());
            } else {
                result.push(letter);
            }
        }
        preceding_lowercase = letter.is_lowercase();
    }
    result
}

fn expand_tiles(text: &str) -> Result<String> {
    let mut expanded = String::new();
    let mut rest = text;
    while let Some((literal, after)) = rest.split_once('^') {
        let (count, next) = after
            .split_once('~')
            .context("Unterminated Pocket City tile run")?;
        let count: usize = count.parse().context("Invalid Pocket City tile run")?;
        ensure!(
            count > 0 && count <= MAX_BYTES / EMPTY_TILE.len(),
            "Invalid Pocket City tile run length"
        );
        ensure!(
            expanded.len() + literal.len() + count * EMPTY_TILE.len() <= MAX_BYTES,
            "Expanded city exceeds 64 MiB"
        );
        expanded.push_str(literal);
        expanded.extend(std::iter::repeat_n(EMPTY_TILE, count));
        rest = next;
    }
    ensure!(
        expanded.len() + rest.len() <= MAX_BYTES,
        "Expanded city exceeds 64 MiB"
    );
    expanded.push_str(rest);
    Ok(expanded)
}

fn pack_tiles(text: &str) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some((literal, after)) = rest.split_once(EMPTY_TILE) {
        result.push_str(literal);
        rest = after;
        let mut count = 1;
        while let Some(after) = rest.strip_prefix(EMPTY_TILE) {
            count += 1;
            rest = after;
        }
        result.push_str(&format!("^{count}~"));
    }
    result.push_str(rest);
    result
}

pub fn file_label(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

pub fn file_game(path: &Path) -> Option<Game> {
    let name = file_label(path);
    if name == "_user.save"
        || (name.starts_with("_city_") || name.starts_with("pocket_city_s_"))
            && name.ends_with(".save")
    {
        Some(Game::One)
    } else if (name.starts_with("city_") && name.ends_with(".es3"))
        || matches!(
            name.as_str(),
            "global_settings" | "survival_global_settings" | "os_settings"
        )
    {
        Some(Game::Two)
    } else {
        None
    }
}

pub fn scan(folder: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_file() && file_game(&path).is_some() {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

pub fn default_folders() -> Vec<PathBuf> {
    let mut folders = Vec::new();
    #[cfg(windows)]
    {
        if let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
            for vendor in ["Codebrew Games Inc_", "Codebrew Games Inc"] {
                folders.push(
                    home.join("AppData/LocalLow")
                        .join(vendor)
                        .join("Pocket City 2/pocketcity2"),
                );
            }
        }
        if let Some(roaming) = std::env::var_os("APPDATA").map(PathBuf::from) {
            for app in ["Pocket City", "com.codebrewgames.pocketcity"] {
                folders.push(roaming.join(app));
            }
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        folders.push(
            home.join("Library/Application Support/Codebrew Games Inc/Pocket City 2/pocketcity2"),
        );
        folders.push(home.join("Library/Application Support/Pocket City"));
    }
    folders
}

#[cfg(test)]
mod tests {
    use super::progress::{QuestState, ResearchState};
    use super::*;

    const PC1: &str = r#"{"AS":[{"timestamp":100,"metadata":{"name":"São Vicente"},"stats":{"money":42,"regions":[^2~""]},"grids":[{_x_{"_a_":1},_y_"_0_"}]}],"MS":[{"timestamp":200,"metadata":{"name":"Nova São Vicente"},"stats":{"money":51},"grids":[]}]}"#;
    const PC2: &str = r#"{"CITY":{"__type":"GameState,Assembly-CSharp","value":{"FILE_ID":"synthetic","money":601,"resources":{0:5,7:20},"terrain":{[2,4]:true},"name":"Example","level":4}}}"#;

    #[test]
    fn pocket_city_edits_the_selected_snapshot_only() -> Result<()> {
        let path = PathBuf::from("_city_example.save");
        let mut save = Save::decode(path, PC1.as_bytes().to_vec())?;
        assert_eq!(save.latest, 1);
        assert_eq!(save.title, "Nova São Vicente");
        save.fields
            .iter_mut()
            .find(|field| field.snapshot == 1 && field.path == "stats.money")
            .context("Missing manual snapshot balance")?
            .value = "90210".to_owned();
        let expected = PC1.replace("\"money\":51", "\"money\":90210");
        assert_eq!(save.output()?, expected.as_bytes());
        assert!(expand_tiles("^999999999~").is_err());
        Ok(())
    }

    #[test]
    fn es3_preserves_opaque_keys_and_gzip_metadata() -> Result<()> {
        let path = PathBuf::from("city_test.es3");
        let mut gzip = GzBuilder::new()
            .mtime(1000)
            .filename("city_test.es3")
            .write(Vec::new(), Compression::default());
        gzip.write_all(PC2.as_bytes())?;
        let bytes = gzip.finish()?;
        let mut save = Save::decode(path, bytes.clone())?;
        assert_eq!(save.output()?, bytes);
        save.fields
            .iter_mut()
            .find(|field| field.path == "resources.7")
            .context("Missing ES3 enum key")?
            .value = "99".to_owned();
        let output = save.output()?;
        let mut gzip = GzDecoder::new(output.as_slice());
        let header = gzip.header().context("Missing gzip header")?;
        assert_eq!(header.filename(), Some(b"city_test.es3".as_slice()));
        assert_eq!(header.mtime(), 1000);
        let mut text = String::new();
        gzip.read_to_string(&mut text)?;
        assert_eq!(text, PC2.replace("7:20", "7:99"));
        Ok(())
    }

    #[test]
    fn protects_existing_files_and_restores_backups() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("city_test.es3");
        fs::write(&path, PC2)?;
        let mut save = Save::open(&path)?;
        save.fields
            .iter_mut()
            .find(|field| field.path == "money")
            .context("Missing balance")?
            .value = "999".to_owned();
        assert!(save.export(&path).is_err());
        let edited = save.save()?;
        let backups = edited.backups()?;
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0])?, PC2.as_bytes());
        assert!(save.save().is_err());
        edited.restore(&backups[0])?;
        assert_eq!(fs::read(&path)?, PC2.as_bytes());
        Ok(())
    }

    #[test]
    fn adds_missing_npc_levels_to_es3_dictionaries() -> Result<()> {
        for existing in [
            None,
            Some("null"),
            Some("{}"),
            Some("{50:42}"),
            Some("{50:42,}"),
        ] {
            let data = existing.map_or_else(
                || PC2.to_owned(),
                |dictionary| {
                    PC2.replace(
                        "\"level\":4",
                        &format!("\"relationships\":{dictionary},\"level\":4"),
                    )
                },
            );
            let path = PathBuf::from("city_test.es3");
            let mut save = Save::decode(path.clone(), data.into_bytes())?;
            let affinity = |field: &&mut Field| field.path == "relationships.1";
            save.fields
                .iter_mut()
                .find(affinity)
                .context("Missing Olivia")?
                .value = "1000".into();
            assert!(save.output().is_err());
            save.fields
                .iter_mut()
                .find(affinity)
                .context("Missing Olivia")?
                .value = "999".into();
            save.fields
                .iter_mut()
                .find(|field| field.path == "relationships.54")
                .context("Missing Eddie")?
                .value = "78".into();
            let rewritten = Save::decode(path, save.output()?)?;
            assert!(
                rewritten
                    .fields
                    .iter()
                    .any(|field| field.path == "relationships.1"
                        && field.value == "999"
                        && field.addition.is_none())
            );
            assert!(
                rewritten
                    .fields
                    .iter()
                    .any(|field| field.path == "relationships.54"
                        && field.value == "78"
                        && field.addition.is_none())
            );
            if existing.is_some_and(|text| text.contains("50:42")) {
                assert!(
                    rewritten
                        .fields
                        .iter()
                        .any(|field| field.path == "relationships.50" && field.value == "42")
                );
            }
        }
        Ok(())
    }

    #[test]
    fn pc1_metadata_is_protected_and_level_remains_editable() -> Result<()> {
        let source = PC1.replace("\"money\":51", "\"money\":51,\"level\":1,\"exp\":400");
        let path = PathBuf::from("_city_test.save");
        let mut save = Save::decode(path.clone(), source.into_bytes())?;
        let metadata = save
            .fields
            .iter_mut()
            .find(|f| f.snapshot == 1 && f.path == "metadata.name")
            .context("No city name")?;
        assert!(metadata.locked(Game::One));
        metadata.value = "Other".into();
        assert!(save.output().is_err());
        let metadata = save
            .fields
            .iter_mut()
            .find(|f| f.snapshot == 1 && f.path == "metadata.name")
            .context("No city name")?;
        metadata.value.clone_from(&metadata.initial);
        let level = save
            .fields
            .iter_mut()
            .find(|f| f.snapshot == 1 && f.path == "stats.level")
            .context("No level")?;
        level.value = "50".into();
        let rewritten = Save::decode(path, save.output()?)?;
        assert!(
            rewritten
                .fields
                .iter()
                .any(|f| f.snapshot == 1 && f.path == "stats.level" && f.value == "50")
        );
        assert_eq!(rewritten.title, "Nova São Vicente");
        Ok(())
    }

    #[test]
    fn pc2_sandbox_flags_and_resource_automation_match_the_game() -> Result<()> {
        let source = PC2.replacen(
            "\"money\":601",
            "\"money\":601,\"isSurvivalMode\":true,\"unlockAll\":false,\"infiniteMoney\":false,\"disableQuests\":false,\"maxLevel\":false",
            1,
        );
        let path = PathBuf::from("city_example.es3");
        let mut save = Save::decode(path.clone(), source.into_bytes())?;
        assert!(save.is_pc2_city() && save.pc2_survival() && !save.pc2_sandbox());
        for key in PC2_SANDBOX_FLAGS {
            let flag = save
                .fields
                .iter_mut()
                .find(|field| field.path == key)
                .context("Missing sandbox flag")?;
            flag.value = "true".into();
            assert!(save.pc2_sandbox());
            assert!(save.output().is_err());
            save.fields
                .iter_mut()
                .find(|field| field.path == key)
                .context("Missing sandbox flag")?
                .value = "false".into();
        }
        let key = "resourceAutoSellKeepAmt.1";
        let stock = save
            .fields
            .iter_mut()
            .find(|field| field.path == key)
            .context("Missing auto-sell stock")?;
        assert_eq!(stock.value, "-1");
        stock.value = "-2".into();
        assert!(save.output().is_err());
        save.fields
            .iter_mut()
            .find(|field| field.path == key)
            .context("Missing auto-sell stock")?
            .value = "120".into();
        let edited = Save::decode(path, save.output()?)?;
        assert!(!edited.pc2_sandbox());
        assert!(
            edited
                .fields
                .iter()
                .any(|field| field.path == key && field.value == "120" && field.addition.is_none())
        );
        Ok(())
    }

    #[test]
    fn pc2_building_flags_keep_progression_and_sandbox_settings() -> Result<()> {
        let source = PC2.replace("\"level\":4", "\"level\":4,\"XP\":2100,\"unlockAll\":false");
        let path = PathBuf::from("city_test.es3");
        let mut save = Save::decode(path.clone(), source.into_bytes())?;
        let (airport, _) = Collection::Buildings
            .catalog()
            .iter()
            .find(|(_, name)| name.starts_with("Airport"))
            .context("No airport")?;
        let key = format!("forceUnlockedBuildings.{airport}");
        for (path, value) in [("relationships.1", "999"), (&key, "true")] {
            save.fields
                .iter_mut()
                .find(|f| f.path == path)
                .context("No field")?
                .value = value.into();
        }
        let mut rewritten = Save::decode(path, save.output()?)?;
        assert!(
            rewritten
                .fields
                .iter()
                .any(|f| f.path == key && f.value == "true" && f.locked(Game::Two))
        );
        assert!(
            rewritten
                .fields
                .iter()
                .any(|f| f.path == "relationships.1" && f.value == "999" && f.addition.is_none())
        );
        assert!(
            rewritten
                .fields
                .iter()
                .any(|f| f.path == "unlockAll" && f.value == "false")
        );
        let level = rewritten
            .fields
            .iter_mut()
            .find(|f| f.path == "level")
            .context("No level")?;
        assert_eq!(level.value, "4");
        level.value = "1".into();
        assert!(rewritten.output().is_err());
        rewritten
            .fields
            .iter_mut()
            .find(|f| f.path == "XP")
            .context("No XP")?
            .value = "0".into();
        assert!(rewritten.output().is_ok());
        Ok(())
    }

    #[test]
    fn pc1_progression_edits_one_snapshot_and_leaves_rewards_claimable() -> Result<()> {
        let source = PC1.replace(
            "\"money\":51",
            "\"money\":51,\"globalSettings\":{\"chest_saved\":{\"10_12\":1}},\"activeQuests\":[1,2],\"newQuests\":[1],\"archivedQuests\":[3]",
        );
        let path = PathBuf::from("_city_test.save");
        let mut save = Save::decode(path.clone(), source.as_bytes().to_vec())?;
        let Some(Progress::Pc1(state)) = &mut save.progress[1] else {
            anyhow::bail!("Missing PC1 progression")
        };
        assert_eq!(state.quest(1), Some(QuestState::Unread));
        state.toggle_quest_claim(1);
        assert_eq!(state.quest(1), Some(QuestState::Claimable));
        state.toggle_quest_claim(1);
        assert_eq!(state.quest(1), Some(QuestState::Unread));
        assert_eq!(save.output()?, source.as_bytes());
        let Some(Progress::Pc1(state)) = &mut save.progress[1] else {
            unreachable!()
        };
        state.toggle_quest_claim(1);
        state.toggle_quest_claim(3);
        state.toggle_policy(1);
        assert_eq!(state.policy(1), Some(false));
        state.toggle_policy(0);
        state.toggle_policy(1);
        state.toggle_policy(0);
        state.unlock_treasure(1);
        save.fields
            .iter_mut()
            .find(|field| field.snapshot == 1 && field.path == "stats.money")
            .context("Missing balance")?
            .value = "52".to_owned();
        let rewritten = Save::decode(path, save.output()?)?;
        let Some(Progress::Pc1(edited)) = &rewritten.progress[1] else {
            anyhow::bail!("Missing rewritten PC1 progression")
        };
        assert_eq!(edited.policy(0), Some(true));
        assert_eq!(edited.policy(1), Some(true));
        assert_eq!(edited.quest(1), Some(QuestState::Claimable));
        assert_eq!(edited.quest(2), Some(QuestState::Active));
        assert_eq!(edited.quest(3), Some(QuestState::Archived));
        assert_eq!(edited.treasure(1), Some(true));
        assert_eq!(rewritten.changed(), 0);
        let (original_auto, _) = source.split_once(",\"MS\"").unwrap();
        assert!(String::from_utf8(rewritten.output()?)?.starts_with(original_auto));
        assert!(rewritten.source.contains("\"chest_saved\":{\"10_12\":1}"));
        assert!(rewritten.source.contains("\"money\":52"));
        assert!(rewritten.source.contains("\"newQuests\":[]"));
        Ok(())
    }

    #[test]
    fn pc2_research_presence_and_pet_following_round_trip_together() -> Result<()> {
        let source = PC2.replace("\"level\":4", "\"level\":4,\"researchPoints\":50,\"researched\":{1:false,2:true,999: true,},\"disabledResearch\":{1:false,2:true,999: false,},\"pets\":{1:1,5:0},\"petStates\":{1:{\"wellBeing\":5,\"lastdayFed\":9,\"lastdayPet\":7},5:{\"wellBeing\":1}},\"eventCooldowns\":{1:0,2:25.7,999:4}");
        let path = PathBuf::from("city_test.es3");
        let mut save = Save::decode(path.clone(), source.into_bytes())?;
        let Some(Progress::Pc2(state)) = &mut save.progress[0] else {
            anyhow::bail!("Missing PC2 progression")
        };
        assert_eq!(state.research(1), Some(ResearchState::Disabled));
        assert!(!state.can_set_research(1, ResearchState::Locked));
        state.set_research(2, ResearchState::Locked);
        state.set_research(1, ResearchState::Active);
        state.set_pet(1, 11);
        assert_eq!(state.pet(1), Some(5));
        state.set_pet(1, 10);
        state.toggle_follower(5);
        assert_eq!(state.cooldown(1), Some((0.0, false)));
        state.toggle_clear_cooldown(1);
        state.toggle_clear_cooldown(2);
        state.toggle_clear_cooldown(2);
        let rewritten = Save::decode(path, save.output()?)?;
        let Some(Progress::Pc2(edited)) = &rewritten.progress[0] else {
            anyhow::bail!("Missing rewritten PC2 progression")
        };
        assert_eq!(edited.research(1), Some(ResearchState::Active));
        assert_eq!(edited.research(2), Some(ResearchState::Locked));
        assert_eq!(edited.pet(1), Some(10));
        assert_eq!(edited.pet_following(1), Some(false));
        assert_eq!(edited.pet_following(5), Some(true));
        assert_eq!(edited.cooldown(1), None);
        assert_eq!(edited.cooldown(2), Some((25.7, false)));
        assert!(
            rewritten
                .source
                .contains("\"researched\":{1:false,999: true}")
        );
        assert!(
            rewritten
                .source
                .contains("\"disabledResearch\":{999: false}")
        );
        assert!(
            rewritten
                .source
                .contains("\"lastdayFed\":9,\"lastdayPet\":7")
        );
        assert!(rewritten.source.contains("\"researchPoints\":50"));
        assert!(
            rewritten
                .source
                .contains("\"eventCooldowns\":{2:25.7,999:4}")
        );
        Ok(())
    }

    #[test]
    fn pc2_research_can_create_missing_and_null_dictionaries() -> Result<()> {
        let path = PathBuf::from("city_test.es3");
        let source = PC2.replace("\"level\":4", "\"level\":4,\"disabledResearch\":null");
        let mut save = Save::decode(path.clone(), source.into_bytes())?;
        let Some(Progress::Pc2(state)) = &mut save.progress[0] else {
            anyhow::bail!("Missing PC2 progression")
        };
        assert!(!state.can_set_research(2, ResearchState::Active));
        state.set_research(1, ResearchState::Disabled);
        state.unlock_all_research();
        let rewritten = Save::decode(path, save.output()?)?;
        let Some(Progress::Pc2(edited)) = &rewritten.progress[0] else {
            anyhow::bail!("Missing rewritten progression")
        };
        assert_eq!(edited.research(1), Some(ResearchState::Disabled));
        for &(id, _) in crate::catalog::RESEARCH.iter().filter(|row| row.0 != 1) {
            assert_eq!(edited.research(id), Some(ResearchState::Active));
        }
        Ok(())
    }

    #[test]
    #[ignore = "uses local Steam save fixtures specified by the environment"]
    fn real_steam_saves() -> Result<()> {
        let directory = tempfile::tempdir()?;
        for (variable, game) in [
            ("PCSE_PC1_FIXTURES", Game::One),
            ("PCSE_PC2_FIXTURES", Game::Two),
        ] {
            let source = std::env::var_os(variable).with_context(|| format!("Set {variable}"))?;
            let paths = scan(Path::new(&source))?;
            ensure!(!paths.is_empty(), "No fixtures in {variable}");
            for path in paths {
                let original = fs::read(&path)?;
                let destination = directory.path().join(file_label(&path));
                fs::write(&destination, &original)?;
                let mut opened = Save::open(&destination)?;
                println!(
                    "{}: {} fields, {} snapshots",
                    file_label(&path),
                    opened.fields.len(),
                    opened.snapshots.len()
                );
                assert_eq!(opened.game, game);
                assert_eq!(opened.output()?, original);
                let latest = opened.latest;
                let balance = if game == Game::One {
                    "stats.money"
                } else {
                    "money"
                };
                if let Some(field) = opened
                    .fields
                    .iter_mut()
                    .find(|field| field.snapshot == latest && field.path == balance)
                {
                    field.value = "1000203".to_owned();
                    let (level_key, xp_key) = game.progression();
                    let mut staged = vec![(level_key.to_owned(), "57"), (xp_key.to_owned(), "0")];
                    if game == Game::Two {
                        staged.push(("relationships.1".to_owned(), "671"));
                        if let Some(building) = opened.fields.iter().find(|field| {
                            Collection::Buildings.matches(&field.path) && !field.locked(game)
                        }) {
                            staged.push((building.path.clone(), "true"));
                        }
                    }
                    for (key, value) in &staged {
                        opened
                            .fields
                            .iter_mut()
                            .find(|field| field.snapshot == latest && field.path == *key)
                            .with_context(|| format!("Missing {key}"))?
                            .value = (*value).to_owned();
                    }
                    match opened.progress[latest].as_mut() {
                        Some(Progress::Pc1(city)) => {
                            city.unlock_treasure(1);
                            if city.policy(0) == Some(false) {
                                city.toggle_policy(0);
                            }
                        }
                        Some(Progress::Pc2(city)) => {
                            city.set_research(1, ResearchState::Active);
                        }
                        None => anyhow::bail!("Missing city progression"),
                    }
                    let rewritten = opened.save()?;
                    assert!(rewritten.fields.iter().any(|field| field.snapshot == latest
                        && field.path == balance
                        && field.value == "1000203"));
                    for (key, value) in staged {
                        assert!(rewritten.fields.iter().any(|field| field.snapshot == latest
                            && field.path == key
                            && field.value == value));
                    }
                    match rewritten.progress[latest].as_ref() {
                        Some(Progress::Pc1(city)) => {
                            assert_eq!(city.treasure(1), Some(true));
                            assert_eq!(city.policy(0), Some(true));
                        }
                        Some(Progress::Pc2(city)) => {
                            assert_eq!(city.research(1), Some(ResearchState::Active));
                        }
                        None => anyhow::bail!("Missing rewritten city progression"),
                    }
                    let backup = rewritten.backups()?.pop().context("No original backup")?;
                    assert_eq!(fs::read(&backup)?, original);
                    rewritten.restore(&backup)?;
                    assert_eq!(fs::read(&destination)?, original);
                }
            }
        }
        Ok(())
    }
}
