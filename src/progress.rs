use super::{Game, Node, Parser, object_end};
use crate::progression_catalog::{
    CHEST_BUILDINGS, EVENTS, PC1_QUESTS, PETS, POLICIES, RESEARCH_PARENTS,
};
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeMap, ops::Range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestState {
    Unavailable,
    Active,
    Unread,
    Claimable,
    Archived,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResearchState {
    Locked,
    Active,
    Disabled,
}

struct Staged<T> {
    original: T,
    pending: T,
}

impl<T: Clone> Staged<T> {
    fn new(value: T) -> Self {
        Self {
            original: value.clone(),
            pending: value,
        }
    }
}

pub struct Pc1 {
    policies: Option<Staged<Vec<i32>>>,
    treasure: Option<Staged<[bool; 3]>>,
    quests: Option<Staged<Quests>>,
}

#[derive(Clone)]
struct Quests {
    active: Vec<i32>,
    unread: Vec<i32>,
    claimable: Vec<i32>,
    archived: Vec<i32>,
}

impl Quests {
    fn status(&self, id: i32) -> QuestState {
        if self.archived.contains(&id) {
            QuestState::Archived
        } else if self.claimable.contains(&id) {
            QuestState::Claimable
        } else if self.unread.contains(&id) && self.active.contains(&id) {
            QuestState::Unread
        } else if self.active.contains(&id) {
            QuestState::Active
        } else {
            QuestState::Unavailable
        }
    }
}

impl Pc1 {
    pub fn policy(&self, id: i32) -> Option<bool> {
        Some(self.policies.as_ref()?.pending.contains(&id))
    }

    pub fn can_set_policy(&self, id: i32) -> bool {
        match self.policy(id) {
            Some(true) => !POLICIES.iter().any(|&(dependent, _, _, parent)| {
                parent == Some(id) && self.policy(dependent) == Some(true)
            }),
            Some(false) => POLICIES.iter().any(|&(policy, _, _, parent)| {
                policy == id && parent.is_none_or(|parent| self.policy(parent) == Some(true))
            }),
            None => false,
        }
    }

    pub fn toggle_policy(&mut self, id: i32) {
        if !self.can_set_policy(id) {
            return;
        }
        if let Some(policies) = self.policies.as_mut() {
            if policies.pending.contains(&id) {
                policies.pending.retain(|&policy| policy != id);
            } else {
                policies.pending.push(id);
            }
        }
    }

    pub fn treasure(&self, index: usize) -> Option<bool> {
        self.treasure.as_ref()?.pending.get(index).copied()
    }

    pub fn unlock_treasure(&mut self, index: usize) {
        if let Some(value) = self
            .treasure
            .as_mut()
            .and_then(|set| set.pending.get_mut(index))
        {
            *value = true;
        }
    }

    pub fn quest(&self, id: i32) -> Option<QuestState> {
        Some(self.quests.as_ref()?.pending.status(id))
    }

    pub fn quest_pending(&self, id: i32) -> bool {
        self.quests
            .as_ref()
            .is_some_and(|quests| quests.original.status(id) != quests.pending.status(id))
    }

    pub fn toggle_quest_claim(&mut self, id: i32) {
        if !PC1_QUESTS.iter().any(|&(known, _)| known == id) {
            return;
        }
        let Some(quests) = &mut self.quests else {
            return;
        };
        if !matches!(
            quests.original.status(id),
            QuestState::Active | QuestState::Unread
        ) {
            return;
        }
        if quests.pending.status(id) == QuestState::Claimable {
            quests.pending.claimable.retain(|&key| key != id);
            restore_quest_order(&mut quests.pending.active, &quests.original.active, id);
            restore_quest_order(&mut quests.pending.unread, &quests.original.unread, id);
        } else {
            quests.pending.active.retain(|&key| key != id);
            quests.pending.unread.retain(|&key| key != id);
            quests.pending.claimable.insert(0, id);
        }
    }
}

fn restore_quest_order(pending: &mut Vec<i32>, original: &[i32], id: i32) {
    if !original.contains(&id) {
        return;
    }
    // The active and unread lists can only lose entries, so their original
    // order can be restored without affecting other staged quest changes.
    pending.retain(|&key| key != id);
    *pending = original
        .iter()
        .copied()
        .filter(|key| *key == id || pending.contains(key))
        .collect();
}

pub struct Pc2 {
    research: Option<Staged<BTreeMap<i32, ResearchState>>>,
    has_disabled_research: bool,
    pets: Staged<BTreeMap<i32, i32>>,
    following: Option<Staged<BTreeMap<i32, bool>>>,
    cooldowns: BTreeMap<i32, (f32, bool)>,
}

impl Pc2 {
    pub fn research(&self, id: i32) -> Option<ResearchState> {
        self.research.as_ref()?.pending.get(&id).copied()
    }

    pub fn can_set_research(&self, id: i32, state: ResearchState) -> bool {
        let Some(current) = self.research(id) else {
            return false;
        };
        if current == state {
            return true;
        }
        if state == ResearchState::Locked {
            return !RESEARCH_PARENTS.iter().any(|&(child, parent)| {
                parent == id
                    && self
                        .research(child)
                        .is_some_and(|value| value != ResearchState::Locked)
            });
        }
        current != ResearchState::Locked
            || RESEARCH_PARENTS
                .iter()
                .find(|&&(child, _)| child == id)
                .is_none_or(|&(_, parent)| {
                    self.research(parent)
                        .is_some_and(|value| value != ResearchState::Locked)
                })
    }

    pub fn set_research(&mut self, id: i32, state: ResearchState) {
        if self.can_set_research(id, state)
            && let Some(research) = &mut self.research
        {
            research.pending.insert(id, state);
        }
    }

    pub fn unlock_all_research(&mut self) {
        loop {
            let eligible = crate::catalog::RESEARCH.iter().find(|&&(id, _)| {
                self.research(id) == Some(ResearchState::Locked)
                    && self.can_set_research(id, ResearchState::Active)
            });
            let Some(&(id, _)) = eligible else { return };
            self.set_research(id, ResearchState::Active);
        }
    }

    pub fn pet(&self, id: i32) -> Option<i32> {
        self.pets.pending.get(&id).copied()
    }

    pub fn set_pet(&mut self, id: i32, wellbeing: i32) {
        if (0..=10).contains(&wellbeing)
            && let Some(value) = self.pets.pending.get_mut(&id)
        {
            *value = wellbeing;
        }
    }

    pub fn pet_following(&self, id: i32) -> Option<bool> {
        self.pet(id)?;
        self.following.as_ref()?.pending.get(&id).copied()
    }

    pub fn toggle_follower(&mut self, id: i32) {
        if let Some(follows) = self.pet_following(id)
            && let Some(following) = &mut self.following
        {
            if !follows {
                following
                    .pending
                    .values_mut()
                    .for_each(|value| *value = false);
            }
            following.pending.insert(id, !follows);
        }
    }

    pub fn cooldown(&self, id: i32) -> Option<(f32, bool)> {
        self.cooldowns.get(&id).copied()
    }

    pub fn toggle_clear_cooldown(&mut self, id: i32) {
        if let Some((_, cleared)) = self.cooldowns.get_mut(&id) {
            *cleared = !*cleared;
        }
    }
}

pub enum Progress {
    Pc1(Pc1),
    Pc2(Pc2),
}

impl Progress {
    pub(super) fn read(game: Game, record: &Node, source: &str) -> Option<Self> {
        let read_list = |node: Option<&Node>| {
            let Some(node) = node else {
                return Some(Vec::new());
            };
            serde_json::from_str::<Option<Vec<i32>>>(&source[node.span.clone()])
                .ok()
                .map(Option::unwrap_or_default)
        };
        let read_object = |node: Option<&Node>| {
            node.is_none_or(|node| {
                let text = &source[node.span.clone()];
                text.starts_with('{') || text == "null"
            })
        };
        match game {
            Game::One => {
                let stats = record.get("stats")?;
                let settings = stats.get("globalSettings");
                let treasure = settings.and_then(|node| node.get("chestunlockbuildings"));
                let treasure = (read_object(settings) && read_object(treasure)).then(|| {
                    Staged::new(std::array::from_fn(|index| {
                        treasure
                            .and_then(|node| node.get(CHEST_BUILDINGS[index].0))
                            .is_some_and(|val| {
                                // The shipped renderer tests the JavaScript truthiness of the reward value.
                                match serde_json::from_str::<serde_json::Value>(
                                    &source[val.span.clone()],
                                ) {
                                    Ok(
                                        serde_json::Value::Null | serde_json::Value::Bool(false),
                                    ) => false,
                                    Ok(serde_json::Value::Number(num)) => num.as_f64() != Some(0.0),
                                    Ok(serde_json::Value::String(text)) => !text.is_empty(),
                                    _ => true,
                                }
                            })
                    }))
                });
                let quests = || {
                    Some(Staged::new(Quests {
                        active: read_list(stats.get("activeQuests"))?,
                        unread: read_list(stats.get("newQuests"))?,
                        claimable: read_list(stats.get("completedQuests"))?,
                        archived: read_list(stats.get("archivedQuests"))?,
                    }))
                };
                Some(Self::Pc1(Pc1 {
                    policies: read_list(stats.get("policies")).map(Staged::new),
                    treasure,
                    quests: quests(),
                }))
            }
            Game::Two => {
                record.get("FILE_ID")?;
                let unlocked = record.get("researched");
                let disabled = record.get("disabledResearch");
                let research = (read_object(unlocked) && read_object(disabled)).then(|| {
                    Staged::new(
                        crate::catalog::RESEARCH
                            .iter()
                            .map(|&(id, _)| {
                                let key = id.to_string();
                                let state = if unlocked.and_then(|node| node.get(&key)).is_none() {
                                    ResearchState::Locked
                                } else if disabled.and_then(|node| node.get(&key)).is_some() {
                                    ResearchState::Disabled
                                } else {
                                    ResearchState::Active
                                };
                                (id, state)
                            })
                            .collect(),
                    )
                });
                let pets = PETS
                    .iter()
                    .filter_map(|&(id, _)| {
                        let key = id.to_string();
                        let state = record.get("pets")?.get(&key)?;
                        source[state.span.clone()]
                            .parse::<i32>()
                            .ok()
                            .filter(|value| matches!(value, 0 | 1))?;
                        let wellbeing = record.get("petStates")?.get(&key)?.get("wellBeing")?;
                        let wellbeing = source[wellbeing.span.clone()].parse::<i32>().ok()?;
                        Some((id, wellbeing))
                    })
                    .collect();
                let following = record.get("pets").and_then(|node| {
                    let values = node
                        .children
                        .iter()
                        .map(|(key, child)| {
                            let id = key.parse::<i32>().ok()?;
                            let follows = match &source[child.span.clone()] {
                                "0" => false,
                                "1" => true,
                                _ => return None,
                            };
                            Some((id, follows))
                        })
                        .collect::<Option<BTreeMap<_, _>>>()?;
                    (values.len() == node.children.len()).then(|| Staged::new(values))
                });
                Some(Self::Pc2(Pc2 {
                    research,
                    has_disabled_research: disabled
                        .is_some_and(|node| source[node.span.clone()].starts_with('{')),
                    pets: Staged::new(pets),
                    following,
                    cooldowns: EVENTS
                        .iter()
                        .filter_map(|&(id, _)| {
                            let value = record.get("eventCooldowns")?.get(&id.to_string())?;
                            let seconds = source[value.span.clone()].parse::<f32>().ok()?;
                            seconds.is_finite().then_some((id, (seconds, false)))
                        })
                        .collect(),
                }))
            }
        }
    }

    pub fn changed(&self) -> usize {
        self.edits().len()
    }

    fn edits(&self) -> Vec<(Vec<String>, Option<String>)> {
        let mut edits = Vec::new();
        let mut set = |path: &[&str], value: Option<String>| {
            edits.push((path.iter().map(|part| (*part).to_owned()).collect(), value));
        };
        match self {
            Self::Pc1(state) => {
                let put_list = |list: &[i32]| Some(format!("{list:?}"));
                if let Some(policies) = &state.policies
                    && policies.pending != policies.original
                {
                    set(&["stats", "policies"], put_list(&policies.pending));
                }
                if let Some(chests) = &state.treasure {
                    for (index, &(key, _)) in CHEST_BUILDINGS.iter().enumerate() {
                        if chests.pending[index] != chests.original[index] {
                            set(
                                &["stats", "globalSettings", "chestunlockbuildings", key],
                                Some("1".to_owned()),
                            );
                        }
                    }
                }
                if let Some(quests) = &state.quests {
                    for (key, original, pending) in [
                        (
                            "activeQuests",
                            &quests.original.active,
                            &quests.pending.active,
                        ),
                        ("newQuests", &quests.original.unread, &quests.pending.unread),
                        (
                            "completedQuests",
                            &quests.original.claimable,
                            &quests.pending.claimable,
                        ),
                    ] {
                        if original != pending {
                            set(&["stats", key], put_list(pending));
                        }
                    }
                }
            }
            Self::Pc2(state) => {
                if let Some(research) = &state.research {
                    if research.pending != research.original
                        && !state.has_disabled_research
                        && !research
                            .pending
                            .values()
                            .any(|value| *value == ResearchState::Disabled)
                    {
                        // Older files can lack this dictionary. The game's active-effect check dereferences it.
                        set(&["disabledResearch"], Some("{}".to_owned()));
                    }
                    for (&id, &pending) in &research.pending {
                        let original = research.original[&id];
                        if pending == original {
                            continue;
                        }
                        let key = id.to_string();
                        if (pending == ResearchState::Locked) != (original == ResearchState::Locked)
                        {
                            set(
                                &["researched", &key],
                                (pending != ResearchState::Locked).then(|| "true".to_owned()),
                            );
                        }
                        // Clear a legacy orphan disable marker when explicitly activating the node.
                        if state.has_disabled_research || pending == ResearchState::Disabled {
                            set(
                                &["disabledResearch", &key],
                                (pending == ResearchState::Disabled).then(|| "true".to_owned()),
                            );
                        }
                    }
                }
                for (&id, &value) in &state.pets.pending {
                    if value != state.pets.original[&id] {
                        set(
                            &["petStates", &id.to_string(), "wellBeing"],
                            Some(value.to_string()),
                        );
                    }
                }
                if let Some(following) = &state.following {
                    for (&id, &value) in &following.pending {
                        if value != following.original[&id] {
                            set(
                                &["pets", &id.to_string()],
                                Some(i32::from(value).to_string()),
                            );
                        }
                    }
                }
                for (&id, &(_, cleared)) in &state.cooldowns {
                    if cleared {
                        set(&["eventCooldowns", &id.to_string()], None);
                    }
                }
            }
        }
        edits
    }
}

#[derive(Default)]
enum Edit {
    #[default]
    Keep,
    Value(Option<String>),
    Object(BTreeMap<String, Edit>),
}

impl Edit {
    fn insert(&mut self, path: &[String], value: Option<String>) {
        if let Some((key, rest)) = path.split_first() {
            if matches!(self, Self::Keep) {
                *self = Self::Object(BTreeMap::new());
            }
            if let Self::Object(children) = self {
                children.entry(key.clone()).or_default().insert(rest, value);
            }
        } else {
            *self = Self::Value(value);
        }
    }

    fn render(&self, node: Option<&Node>, source: &str, game: Game) -> Result<Option<String>> {
        let original = node.map(|node| &source[node.span.clone()]);
        let Self::Object(edits) = self else {
            return Ok(match self {
                Self::Keep => original.map(str::to_owned),
                Self::Value(value) => value.clone(),
                Self::Object(_) => unreachable!(),
            });
        };
        let mut additions = Vec::new();
        let mut replacements = Vec::new();
        let mut deletions = Vec::new();
        for (key, edit) in edits {
            ensure!(
                node.is_none_or(|node| node
                    .children
                    .iter()
                    .filter(|(name, _)| name == key)
                    .count()
                    <= 1),
                "Duplicate progression key {key}"
            );
            let member = node.and_then(|node| node.get(key));
            let rendered = edit.render(member, source, game)?;
            match (member, rendered) {
                (Some(child), Some(text)) if source[child.span.clone()] != text => {
                    replacements.push((child.span.clone(), text));
                }
                (Some(child), None) => deletions.push(child.span.start),
                (None, Some(text)) => {
                    let key = if game == Game::Two && key.parse::<i32>().is_ok() {
                        key.clone()
                    } else {
                        serde_json::to_string(key)?
                    };
                    additions.push(format!("{key}:{text}"));
                }
                _ => {}
            }
        }
        if replacements.is_empty() && additions.is_empty() && deletions.is_empty() {
            return Ok(original.map(str::to_owned));
        }
        let Some(node) = node.filter(|_| original != Some("null")) else {
            return Ok(Some(format!("{{{}}}", additions.join(","))));
        };
        let original = original.context("Missing progression object")?;
        ensure!(
            original.starts_with('{'),
            "Expected an object in progression data"
        );
        if !deletions.is_empty() {
            let mut members = Vec::new();
            for (_, child) in &node.children {
                if deletions.contains(&child.span.start) {
                    continue;
                }
                let prefix = &source[child.key_start..child.span.start];
                let value = replacements
                    .iter()
                    .find(|(range, _)| *range == child.span)
                    .map_or(&source[child.span.clone()], |(_, text)| text.as_str());
                members.push(format!("{prefix}{value}"));
            }
            members.append(&mut additions);
            return Ok(Some(format!("{{{}}}", members.join(","))));
        }
        if !additions.is_empty() {
            let (range, comma) = object_end(node, source);
            replacements.push((
                range,
                format!("{}{}", if comma { "," } else { "" }, additions.join(",")),
            ));
        }
        Ok(Some(rewrite(original, node.span.start, replacements)))
    }
}

fn rewrite(source: &str, offset: usize, mut replacements: Vec<(Range<usize>, String)>) -> String {
    let mut result = source.to_owned();
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    for (range, value) in replacements {
        result.replace_range(range.start - offset..range.end - offset, &value);
    }
    result
}

pub(super) fn apply(source: &str, game: Game, snapshots: &[Option<Progress>]) -> Result<String> {
    let root = Parser {
        source,
        position: usize::from(source.starts_with('\u{feff}')) * 3,
    }
    .value(0)?;
    let records = match game {
        Game::One => root
            .get("AS")
            .into_iter()
            .chain(root.get("MS"))
            .flat_map(|list| list.children.iter().map(|(_, record)| record))
            .collect::<Vec<_>>(),
        Game::Two => root
            .get("CITY")
            .and_then(|city| city.get("value"))
            .into_iter()
            .collect(),
    };
    ensure!(
        records.len() == snapshots.len(),
        "Progression snapshot count changed"
    );
    let mut replacements = Vec::new();
    for (record, progress) in records.into_iter().zip(snapshots) {
        if let Some(progress) = progress {
            let mut edit = Edit::Keep;
            for (path, value) in progress.edits() {
                edit.insert(&path, value);
            }
            if !matches!(edit, Edit::Keep) {
                let updated = edit
                    .render(Some(record), source, game)?
                    .context("Missing rewritten progression")?;
                replacements.push((record.span.clone(), updated));
            }
        }
    }
    Ok(rewrite(source, 0, replacements))
}
