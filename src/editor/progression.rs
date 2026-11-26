use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    catalog::RESEARCH,
    locale::Locale,
    progression_catalog::{CHEST_BUILDINGS, EVENTS, PC1_QUESTS, PETS, POLICIES, RESEARCH_PARENTS},
    save::progress::{
        PC1_TAX_PRESETS, PC2_MAX_TAX_PERCENT, Progress, QuestState, ResearchState, TAX_LABELS,
    },
};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ResearchChoice {
    Locked,
    Active,
    Disabled,
}

impl ResearchChoice {
    fn state(self) -> ResearchState {
        match self {
            Self::Locked => ResearchState::Locked,
            Self::Active => ResearchState::Active,
            Self::Disabled => ResearchState::Disabled,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Locked => "research_locked",
            Self::Active => "research_active",
            Self::Disabled => "research_inactive",
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProgressEdit {
    Treasure { index: usize },
    Policy { id: i32 },
    Quest { id: i32 },
    Research { id: i32, choice: ResearchChoice },
    ResearchAll,
    Pet { id: i32, value: i32 },
    Follower { id: i32 },
    Cooldown { id: i32 },
    TaxPreset { index: usize, preset: usize },
    TaxPercent { index: usize, value: u8 },
    Debt,
}

impl ProgressEdit {
    pub fn apply(self, progress: &mut Progress) -> Result<()> {
        match (self, progress) {
            (Self::Treasure { index }, Progress::Pc1(city)) => city.unlock_treasure(index),
            (Self::Policy { id }, Progress::Pc1(city)) => city.toggle_policy(id),
            (Self::Quest { id }, Progress::Pc1(city)) => city.toggle_quest_claim(id),
            (Self::Research { id, choice }, Progress::Pc2(city)) => {
                city.set_research(id, choice.state())
            }
            (Self::ResearchAll, Progress::Pc2(city)) => city.unlock_all_research(),
            (Self::Pet { id, value }, Progress::Pc2(city)) => city.set_pet(id, value),
            (Self::Follower { id }, Progress::Pc2(city)) => city.toggle_follower(id),
            (Self::Cooldown { id }, Progress::Pc2(city)) => city.toggle_clear_cooldown(id),
            (Self::TaxPreset { index, preset }, progress @ Progress::Pc1(_))
                if index < TAX_LABELS.len() =>
            {
                progress.set_tax_preset(index, preset);
            }
            (Self::TaxPercent { index, value }, progress @ Progress::Pc2(_))
                if index < TAX_LABELS.len() =>
            {
                progress.set_tax_percent(index, value);
            }
            (Self::Debt, progress) => progress.toggle_clear_debt(),
            _ => bail!("This action is unavailable for the open save"),
        }
        Ok(())
    }
}

fn action(label: impl Into<String>, edit: ProgressEdit, selected: bool, enabled: bool) -> Value {
    json!({ "label": label.into(), "edit": edit, "selected": selected, "enabled": enabled })
}

fn row(id: i32, name: &str, help: &str, actions: Vec<Value>) -> Value {
    json!({ "id": id, "name": name, "help": help, "actions": actions })
}

fn section(locale: Locale, title: &str, help: &str, rows: Vec<Value>, empty: &str) -> Value {
    json!({ "key": title, "label": locale.text(title), "help": locale.text(help), "rows": rows, "empty": locale.text(empty) })
}

pub(super) fn view(progress: &Progress, locale: Locale) -> Vec<Value> {
    let t = |key: &str| locale.get(key).unwrap_or_default();
    let mut sections = match progress {
        Progress::Pc1(city) => {
            let treasures = CHEST_BUILDINGS
                .iter()
                .enumerate()
                .filter_map(|(index, &(_, key))| {
                    let unlocked = city.treasure(index)?;
                    let actions = if unlocked {
                        Vec::new()
                    } else {
                        vec![action(
                            t("unlock_one"),
                            ProgressEdit::Treasure { index },
                            false,
                            true,
                        )]
                    };
                    Some(row(
                        [33, 37, 48][index],
                        t(key),
                        if unlocked { t("unlocked_state") } else { "" },
                        actions,
                    ))
                })
                .collect();
            let policy_name =
                |id, fallback| locale.get(&format!("pc1_policy_{id}")).unwrap_or(fallback);
            let policies = POLICIES
                .iter()
                .filter_map(|&(id, name, cost, parent)| {
                    let enabled = city.policy(id)?;
                    let name = policy_name(id, name);
                    let mut help = t("policy_cost").replace("{n}", &cost.to_string());
                    if let Some(parent) = parent
                        && let Some(&(_, fallback, _, _)) =
                            POLICIES.iter().find(|policy| policy.0 == parent)
                    {
                        help.push_str(" · ");
                        help.push_str(
                            &t("requires_name").replace("{name}", policy_name(parent, fallback)),
                        );
                    }
                    let label = t(if enabled {
                        "remove_policy"
                    } else {
                        "approve_policy"
                    });
                    let actions = vec![action(
                        label,
                        ProgressEdit::Policy { id },
                        enabled,
                        city.can_set_policy(id),
                    )];
                    Some(row(id, name, &help, actions))
                })
                .collect();
            let quests = PC1_QUESTS
                .iter()
                .filter_map(|&(id, name)| {
                    let state = city.quest(id)?;
                    let status = t(match state {
                        QuestState::Unavailable => "quest_unavailable",
                        QuestState::Active => "quest_active",
                        QuestState::Unread => "quest_unread",
                        QuestState::Claimable => "quest_claimable",
                        QuestState::Archived => "quest_archived",
                    });
                    let pending = city.quest_pending(id);
                    let actions =
                        if pending || matches!(state, QuestState::Active | QuestState::Unread) {
                            let label = t(if pending {
                                "undo_change"
                            } else {
                                "ready_for_claim"
                            });
                            vec![action(label, ProgressEdit::Quest { id }, false, true)]
                        } else {
                            Vec::new()
                        };
                    Some(row(
                        id,
                        locale.get(&format!("pc1_quest_{id}")).unwrap_or(name),
                        status,
                        actions,
                    ))
                })
                .collect();
            vec![
                section(
                    locale,
                    "treasure_unlocks",
                    "treasure_help",
                    treasures,
                    "empty_fields",
                ),
                section(
                    locale,
                    "city_policies",
                    "policy_help",
                    policies,
                    "empty_fields",
                ),
                section(
                    locale,
                    "city_quests",
                    "pc1_quest_help",
                    quests,
                    "empty_fields",
                ),
            ]
        }
        Progress::Pc2(city) => {
            let research = RESEARCH
                .iter()
                .filter_map(|&(id, name)| {
                    let current = city.research(id)?;
                    let help = RESEARCH_PARENTS
                        .iter()
                        .find(|&&(child, _)| child == id)
                        .and_then(|&(_, parent)| RESEARCH.iter().find(|item| item.0 == parent))
                        .map(|&(_, name)| t("requires_name").replace("{name}", name))
                        .unwrap_or_default();
                    let actions = [
                        ResearchChoice::Locked,
                        ResearchChoice::Active,
                        ResearchChoice::Disabled,
                    ]
                    .into_iter()
                    .map(|choice| {
                        let state = choice.state();
                        action(
                            t(choice.label()),
                            ProgressEdit::Research { id, choice },
                            current == state,
                            city.can_set_research(id, state),
                        )
                    })
                    .collect();
                    Some(row(id, name, &help, actions))
                })
                .collect();
            let mut research = section(
                locale,
                "research",
                "research_state_help",
                research,
                "empty_fields",
            );
            research["bulk"] = action(
                t("complete_research"),
                ProgressEdit::ResearchAll,
                false,
                RESEARCH
                    .iter()
                    .any(|&(id, _)| city.research(id) == Some(ResearchState::Locked)),
            );

            let pets = PETS
                .iter()
                .filter_map(|&(id, key)| {
                    let value = city.pet(id)?;
                    let following = city.pet_following(id);
                    let mut actions = Vec::new();
                    let mut help = "";
                    if let Some(follows) = following {
                        help = t(if follows {
                            "pet_following"
                        } else {
                            "pet_at_home"
                        });
                        let label = t(if follows {
                            "pet_send_home"
                        } else {
                            "pet_follow_mayor"
                        });
                        actions.push(action(label, ProgressEdit::Follower { id }, false, true));
                    }
                    let mut view = row(id, t(key), help, actions);
                    view["slider"] = slider(
                        t("pet_wellbeing"),
                        10,
                        "",
                        ProgressEdit::Pet { id, value },
                        value,
                    );
                    Some(view)
                })
                .collect();
            let cooldowns = EVENTS
                .iter()
                .filter_map(|&(id, name)| {
                    let (seconds, cleared) = city.cooldown(id)?;
                    let help = if cleared {
                        t("cooldown_clear_pending").to_owned()
                    } else {
                        t("cooldown_remaining")
                            .replace("{n}", &format!("{:.0}", seconds.max(0.0).ceil()))
                    };
                    let label = t(if cleared {
                        "undo_change"
                    } else {
                        "clear_cooldown"
                    });
                    Some(row(
                        id,
                        name,
                        &help,
                        vec![action(label, ProgressEdit::Cooldown { id }, false, true)],
                    ))
                })
                .collect();
            vec![
                research,
                section(locale, "city_pets", "pet_help", pets, "no_editable_pets"),
                section(
                    locale,
                    "event_cooldowns",
                    "event_cooldowns_help",
                    cooldowns,
                    "no_event_cooldowns",
                ),
            ]
        }
    };
    sections.push(economy(progress, locale));
    sections
}

fn slider(label: &str, max: i32, unit: &str, edit: ProgressEdit, value: i32) -> Value {
    json!({ "label": label, "max": max, "unit": unit, "edit": edit, "value": value })
}

fn economy(progress: &Progress, locale: Locale) -> Value {
    let mut rows = TAX_LABELS
        .into_iter()
        .enumerate()
        .filter_map(|(index, key)| {
            let rate = progress.tax(index)?;
            let label = locale.text(key);
            let index_id = index as i32;
            if matches!(progress, Progress::Pc1(_)) {
                let actions = PC1_TAX_PRESETS
                    .iter()
                    .enumerate()
                    .map(|(preset, &value)| {
                        action(
                            format!("{:.0}%", value * 100.0),
                            ProgressEdit::TaxPreset { index, preset },
                            rate == value,
                            true,
                        )
                    })
                    .collect();
                Some(row(index_id, label, "", actions))
            } else {
                let value = (rate * 100.0).round() as i32;
                let mut view = row(index_id, label, "", Vec::new());
                view["slider"] = slider(
                    label,
                    PC2_MAX_TAX_PERCENT.into(),
                    "%",
                    ProgressEdit::TaxPercent {
                        index,
                        value: value.clamp(0, i32::from(PC2_MAX_TAX_PERCENT)) as u8,
                    },
                    value,
                );
                Some(view)
            }
        })
        .collect::<Vec<_>>();
    if let Some((principal, cleared)) = progress.debt() {
        let label = locale.text(if cleared { "undo_change" } else { "clear_debt" });
        let mut debt = row(
            -1,
            locale.text("loan"),
            locale.text("debt_help"),
            vec![action(label, ProgressEdit::Debt, false, principal > 0)],
        );
        debt["amount"] = json!(if cleared { 0 } else { principal }.to_string());
        rows.push(debt);
    }
    let help = if matches!(progress, Progress::Pc1(_)) {
        "pc1_tax_help"
    } else {
        "pc2_tax_help"
    };
    section(locale, "economy", help, rows, "empty_fields")
}
