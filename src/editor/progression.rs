use super::*;
use crate::{
    catalog::RESEARCH,
    progression_catalog::{CHEST_BUILDINGS, EVENTS, PC1_QUESTS, PETS, POLICIES, RESEARCH_PARENTS},
    save::progress::{
        PC1_TAX_PRESETS, PC2_MAX_TAX_PERCENT, Progress, QuestState, ResearchState, TAX_LABELS,
    },
};

fn state_label(locale: Locale, state: ResearchState) -> &'static str {
    locale
        .get(match state {
            ResearchState::Locked => "research_locked",
            ResearchState::Active => "research_active",
            ResearchState::Disabled => "research_inactive",
        })
        .unwrap()
}

impl Editor {
    fn progress(&self) -> Option<&Progress> {
        self.current.as_ref()?.progress.get(self.snapshot)?.as_ref()
    }

    fn progress_mut(&mut self) -> Option<&mut Progress> {
        self.current
            .as_mut()?
            .progress
            .get_mut(self.snapshot)?
            .as_mut()
    }

    pub(super) fn progression(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        let Some(progress) = self.progress() else {
            return column();
        };
        let sections = match progress {
            Progress::Pc1(_) => [
                ("treasure_unlocks", "treasure_help"),
                ("city_policies", "policy_help"),
                ("city_quests", "pc1_quest_help"),
                ("economy", "pc1_tax_help"),
            ],
            Progress::Pc2(_) => [
                ("research", "research_state_help"),
                ("city_pets", "pet_help"),
                ("event_cooldowns", "event_cooldowns_help"),
                ("economy", "pc2_tax_help"),
            ],
        };
        let locale = self.locale;
        let mut panel = column()
            .child(
                TabBar::new("progression-tabs")
                    .segmented()
                    .selected_index(self.section)
                    .children(
                        sections
                            .iter()
                            .map(|&(label, _)| KitTab::new().label(locale.text(label))),
                    )
                    .on_click(cx.listener(|this, index: &usize, window, cx| {
                        this.section = *index;
                        this.query
                            .update(cx, |input, cx| input.set_value("", window, cx));
                        cx.notify();
                    })),
            )
            .child(subtle(locale.text(sections[self.section].1)));

        if self.section == 3 {
            return panel.child(self.economy(cx));
        }
        panel = panel.child(Input::new(&self.query).w(px(300.0)));
        let query = self.query.read(cx).value().trim().to_lowercase();
        let includes = |name: &str, id: i32| {
            query.is_empty()
                || name.to_lowercase().contains(&query)
                || id.to_string().contains(&query)
        };
        let mut count = 0;
        match (progress, self.section) {
            (Progress::Pc1(city), 0) => {
                for (index, &(_, label)) in CHEST_BUILDINGS.iter().enumerate() {
                    let name = locale.text(label);
                    if !includes(name, [33, 37, 48][index]) {
                        continue;
                    }
                    if let Some(value) = city.treasure(index) {
                        count += 1;
                        panel = panel.child(
                            row(
                                name,
                                if value {
                                    locale.text("unlocked_state")
                                } else {
                                    ""
                                },
                            )
                            .when(!value, |row| {
                                row.child(action(
                                    cx,
                                    ("treasure", index),
                                    locale.text("unlock_one"),
                                    false,
                                    true,
                                    move |this, _, _| {
                                        if let Some(Progress::Pc1(city)) = this.progress_mut() {
                                            city.unlock_treasure(index);
                                        }
                                    },
                                ))
                            }),
                        );
                    }
                }
            }
            (Progress::Pc1(city), 1) => {
                for &(id, name, cost, parent) in POLICIES {
                    let name = locale.get(&format!("pc1_policy_{id}")).unwrap_or(name);
                    if !includes(name, id) {
                        continue;
                    }
                    if let Some(enabled) = city.policy(id) {
                        count += 1;
                        let mut help = locale.text("policy_cost").replace("{n}", &cost.to_string());
                        if let Some(parent) = parent
                            && let Some((_, label, _, _)) =
                                POLICIES.iter().find(|row| row.0 == parent)
                        {
                            let name = locale.get(&format!("pc1_policy_{parent}")).unwrap_or(label);
                            help.push_str(" · ");
                            help.push_str(&locale.text("requires_name").replace("{name}", name));
                        }
                        panel = panel.child(row(name, &help).child(action(
                            cx,
                            ("policy", id as usize),
                            locale.text(if enabled {
                                "remove_policy"
                            } else {
                                "approve_policy"
                            }),
                            enabled,
                            city.can_set_policy(id),
                            move |this, _, _| {
                                if let Some(Progress::Pc1(city)) = this.progress_mut() {
                                    city.toggle_policy(id);
                                }
                            },
                        )));
                    }
                }
            }
            (Progress::Pc1(city), _) => {
                for &(id, name) in PC1_QUESTS {
                    let name = locale.get(&format!("pc1_quest_{id}")).unwrap_or(name);
                    if !includes(name, id) {
                        continue;
                    }
                    if let Some(state) = city.quest(id) {
                        count += 1;
                        let status = locale.text(match state {
                            QuestState::Unavailable => "quest_unavailable",
                            QuestState::Active => "quest_active",
                            QuestState::Unread => "quest_unread",
                            QuestState::Claimable => "quest_claimable",
                            QuestState::Archived => "quest_archived",
                        });
                        let pending = city.quest_pending(id);
                        panel = panel.child(row(name, status).when(
                            pending || matches!(state, QuestState::Active | QuestState::Unread),
                            |row| {
                                row.child(action(
                                    cx,
                                    ("quest", id as usize),
                                    locale.text(if pending {
                                        "undo_change"
                                    } else {
                                        "ready_for_claim"
                                    }),
                                    false,
                                    true,
                                    move |this, _, _| {
                                        if let Some(Progress::Pc1(city)) = this.progress_mut() {
                                            city.toggle_quest_claim(id);
                                        }
                                    },
                                ))
                            },
                        ));
                    }
                }
            }
            (Progress::Pc2(city), 0) => {
                let enabled = RESEARCH
                    .iter()
                    .any(|&(id, _)| city.research(id) == Some(ResearchState::Locked));
                panel = panel.child(div().flex().child(action(
                    cx,
                    "all-research",
                    locale.text("complete_research"),
                    false,
                    enabled,
                    |this, _, _| {
                        if let Some(Progress::Pc2(city)) = this.progress_mut() {
                            city.unlock_all_research();
                        }
                    },
                )));
                for &(id, name) in RESEARCH {
                    if !includes(name, id) {
                        continue;
                    }
                    if let Some(current) = city.research(id) {
                        count += 1;
                        let parent = RESEARCH_PARENTS
                            .iter()
                            .find(|&&(child, _)| child == id)
                            .and_then(|&(_, parent)| RESEARCH.iter().find(|item| item.0 == parent))
                            .map(|&(_, name)| locale.text("requires_name").replace("{name}", name))
                            .unwrap_or_default();
                        panel = panel.child(
                            row(name, &parent).child(
                                div().flex().flex_wrap().gap_1().children(
                                    [
                                        ResearchState::Locked,
                                        ResearchState::Active,
                                        ResearchState::Disabled,
                                    ]
                                    .into_iter()
                                    .enumerate()
                                    .map(|(option, state)| {
                                        action(
                                            cx,
                                            ("research-state", id as usize * 3 + option),
                                            state_label(locale, state),
                                            current == state,
                                            city.can_set_research(id, state),
                                            move |this, _, _| {
                                                if let Some(Progress::Pc2(city)) =
                                                    this.progress_mut()
                                                {
                                                    city.set_research(id, state);
                                                }
                                            },
                                        )
                                    }),
                                ),
                            ),
                        );
                    }
                }
            }
            (Progress::Pc2(city), 1) => {
                let pets = PETS
                    .iter()
                    .filter_map(|&(id, key)| {
                        let title = locale.text(key);
                        if !includes(title, id) {
                            return None;
                        }
                        Some((id, title, city.pet(id)?, city.pet_following(id)))
                    })
                    .collect::<Vec<_>>();
                for (id, title, value, following) in pets {
                    count += 1;
                    let mut item = row(title, "").child(self.slider(
                        SliderKey::Pet(id),
                        value,
                        10.0,
                        locale.text("pet_wellbeing"),
                        cx,
                    ));
                    if let Some(follows) = following {
                        item = item.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(subtle(locale.text(if follows {
                                    "pet_following"
                                } else {
                                    "pet_at_home"
                                })))
                                .child(action(
                                    cx,
                                    ("pet-follows", id as usize),
                                    locale.text(if follows {
                                        "pet_send_home"
                                    } else {
                                        "pet_follow_mayor"
                                    }),
                                    false,
                                    true,
                                    move |this, _, _| {
                                        if let Some(Progress::Pc2(city)) = this.progress_mut() {
                                            city.toggle_follower(id);
                                        }
                                    },
                                )),
                        );
                    }
                    panel = panel.child(item);
                }
            }
            (Progress::Pc2(city), _) => {
                for &(id, name) in EVENTS {
                    if !includes(name, id) {
                        continue;
                    }
                    if let Some((seconds, cleared)) = city.cooldown(id) {
                        count += 1;
                        let info = if cleared {
                            locale.text("cooldown_clear_pending").to_owned()
                        } else {
                            locale
                                .text("cooldown_remaining")
                                .replace("{n}", &format!("{:.0}", seconds.max(0.0).ceil()))
                        };
                        panel = panel.child(row(name, &info).child(action(
                            cx,
                            ("cooldown", id as usize),
                            locale.text(if cleared {
                                "undo_change"
                            } else {
                                "clear_cooldown"
                            }),
                            false,
                            true,
                            move |this, _, _| {
                                if let Some(Progress::Pc2(city)) = this.progress_mut() {
                                    city.toggle_clear_cooldown(id);
                                }
                            },
                        )));
                    }
                }
            }
        }
        if count == 0 {
            let label = match (
                self.current.as_ref().map(|doc| doc.game),
                self.section,
                query.is_empty(),
            ) {
                (Some(Game::Two), 1, true) => "no_editable_pets",
                (Some(Game::Two), 2, true) => "no_event_cooldowns",
                _ => "empty_fields",
            };
            panel = panel.child(subtle(locale.text(label)));
        }

        panel
    }

    fn economy(&mut self, cx: &mut Context<Self>) -> Div {
        let locale = self.locale;
        let progress = self.progress().unwrap();
        let rates = std::array::from_fn::<_, 3, _>(|index| progress.tax(index));
        let debt = progress.debt();
        let game = self.current.as_ref().unwrap().game;
        let mut taxes = h_form().label_width(px(180.0));
        for (index, key) in TAX_LABELS.into_iter().enumerate() {
            let Some(rate) = rates[index] else { continue };
            let name = locale.text(key);
            let control = if game == Game::One {
                div().flex().flex_wrap().gap_1().children(
                    PC1_TAX_PRESETS
                        .into_iter()
                        .enumerate()
                        .map(|(preset, value)| {
                            action(
                                cx,
                                ("tax-preset", index * 5 + preset),
                                format!("{:.0}%", value * 100.0),
                                value == rate,
                                true,
                                move |this, _, _| {
                                    if let Some(progress) = this.progress_mut() {
                                        progress.set_tax_preset(index, preset);
                                    }
                                },
                            )
                        }),
                )
            } else {
                self.slider(
                    SliderKey::Tax(index),
                    (rate * 100.0).round() as i32,
                    f32::from(PC2_MAX_TAX_PERCENT),
                    name,
                    cx,
                )
            };
            taxes = taxes.child(form::field().label(name).items_center().child(control));
        }
        let mut body = div()
            .flex()
            .flex_col()
            .gap_4()
            .max_w(px(770.0))
            .w_full()
            .child(
                GroupBox::new()
                    .id("city-taxes")
                    .outline()
                    .w_full()
                    .child(taxes),
            );
        if let Some((principal, cleared)) = debt {
            let amount = format!("{}{}", '$', if cleared { 0 } else { principal });
            let repayment = action(
                cx,
                "clear-debt",
                locale.text(if cleared { "undo_change" } else { "clear_debt" }),
                false,
                principal > 0,
                |this, _, _| {
                    if let Some(progress) = this.progress_mut() {
                        progress.toggle_clear_debt();
                    }
                },
            );
            body = body.child(
                GroupBox::new().id("city-debt").outline().w_full().child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .flex_wrap()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .font_semibold()
                                                .child(Label::new(locale.text("loan"))),
                                        )
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_semibold()
                                                .child(Label::new(amount)),
                                        ),
                                )
                                .child(subtle(locale.text("debt_help"))),
                        )
                        .child(repayment),
                ),
            );
        }
        body
    }

    fn slider(
        &mut self,
        key: SliderKey,
        value: i32,
        maximum: f32,
        name: &str,
        cx: &mut Context<Self>,
    ) -> Div {
        let state = &self
            .sliders
            .entry(key)
            .or_insert_with(|| {
                let state = cx.new(|_| {
                    SliderState::new()
                        .min(0.0)
                        .max(maximum)
                        .step(1.0)
                        .default_value(value as f32)
                });
                let subscription = cx.subscribe(&state, move |this, _, event: &SliderEvent, cx| {
                    let SliderEvent::Change(value) = event else {
                        return;
                    };
                    let value = value.start().round() as i32;
                    match key {
                        SliderKey::Tax(index) => {
                            if let Some(progress) = this.progress_mut()
                                && let Ok(percent) = u8::try_from(value)
                            {
                                progress.set_tax_percent(index, percent);
                            }
                        }
                        SliderKey::Pet(id) => {
                            if let Some(Progress::Pc2(city)) = this.progress_mut() {
                                city.set_pet(id, value);
                            }
                        }
                    }
                    cx.notify();
                });
                (state, subscription)
            })
            .0;
        let unit = if matches!(key, SliderKey::Tax(_)) {
            "%"
        } else {
            ""
        };
        let end = format!("{maximum:.0}{unit}");
        div()
            .w_full()
            .max_w(px(440.0))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div().flex_1().child(
                            Slider::new(state)
                                .accessibility_label(name.to_owned())
                                .bg(rgb(accent(Game::Two)))
                                .text_color(rgb(INK)),
                        ),
                    )
                    .child(
                        div()
                            .min_w(px(58.0))
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(BG))
                            .text_center()
                            .child(Label::new(format!("{value}{unit}"))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .pr_16()
                    .child(subtle(format!("0{unit}")))
                    .child(subtle(if matches!(key, SliderKey::Pet(_)) {
                        name.to_owned()
                    } else {
                        end.clone()
                    }))
                    .when(matches!(key, SliderKey::Pet(_)), |row| {
                        row.child(subtle(end))
                    }),
            )
    }
}
