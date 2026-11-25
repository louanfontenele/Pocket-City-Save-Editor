use std::{collections::HashMap, sync::OnceLock};

const CATALOG: &str = include_str!("locales.txt");
const COUNT: usize = 13;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Locale(usize);

impl Locale {
    pub const ALL: [(&str, &str); COUNT] = [
        ("pt-BR", "Português (Brasil)"),
        ("en", "English"),
        ("ja", "日本語"),
        ("fr", "Français"),
        ("it", "Italiano"),
        ("de", "Deutsch"),
        ("es-ES", "Español (España)"),
        ("cs", "Čeština"),
        ("ko", "한국어"),
        ("ru", "Русский"),
        ("zh-Hans", "简体中文"),
        ("es-419", "Español (Latinoamérica)"),
        ("uk", "Українська"),
    ];

    pub fn from_code(code: &str) -> Self {
        Self(
            Self::ALL
                .iter()
                .position(|(tag, _)| *tag == code)
                .unwrap_or_default(),
        )
    }

    pub fn code(self) -> &'static str {
        Self::ALL[self.0].0
    }

    pub fn name(self) -> &'static str {
        Self::ALL[self.0].1
    }

    pub fn get(self, key: &str) -> Option<&'static str> {
        catalog().get(key).map(|row| row[self.0])
    }

    pub fn text(self, key: &str) -> &str {
        self.get(key).unwrap_or(key)
    }

    pub fn install_fonts(self, ctx: &eframe::egui::Context) {
        use eframe::egui::{FontData, FontDefinitions, FontFamily};
        use std::path::PathBuf;

        let mut fonts = FontDefinitions::default();
        let mut families = Vec::new();
        if let Some(windows) = std::env::var_os("WINDIR") {
            let directory = PathBuf::from(windows).join("Fonts");
            let cjk = match self.code() {
                "zh-Hans" => ["msyh.ttc", "YuGothR.ttc", "malgun.ttf"],
                "ko" => ["malgun.ttf", "YuGothR.ttc", "msyh.ttc"],
                _ => ["YuGothR.ttc", "msyh.ttc", "malgun.ttf"],
            };
            for file in std::iter::once("segoeui.ttf").chain(cjk) {
                if let Ok(bytes) = std::fs::read(directory.join(file)) {
                    fonts
                        .font_data
                        .insert(file.into(), FontData::from_owned(bytes).into());
                    families.push(file.to_owned());
                }
            }
        } else {
            for path in [
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
                "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            ] {
                if let Ok(bytes) = std::fs::read(path) {
                    let mut font = FontData::from_owned(bytes);
                    if path.contains("NotoSansCJK") {
                        font.index = match self.code() {
                            "ko" => 1,
                            "zh-Hans" => 2,
                            _ => 0,
                        };
                    }
                    fonts.font_data.insert(path.into(), font.into());
                    families.push(path.to_owned());
                }
            }
        }
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .splice(0..0, families);
        ctx.set_fonts(fonts);
    }
}

fn catalog() -> &'static HashMap<&'static str, [&'static str; COUNT]> {
    static STRINGS: OnceLock<HashMap<&'static str, [&'static str; COUNT]>> = OnceLock::new();
    STRINGS.get_or_init(|| {
        CATALOG
            .lines()
            .map(|line| {
                let (key, values) = line.split_once('|').expect("localization key");
                let values = values.split('|').collect::<Vec<_>>();
                let values = values
                    .try_into()
                    .expect("one translation per supported locale");
                (key, values)
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_rows_cover_every_language() {
        let strings = catalog();
        assert_eq!(strings.len(), CATALOG.lines().count());
        assert!(strings.values().flatten().all(|value| !value.is_empty()));
        for (code, name) in Locale::ALL {
            let locale = Locale::from_code(code);
            assert_eq!(locale.name(), name);
            assert!(locale.text("xp_progress").contains("{xp}"));
            assert!(locale.text("xp_progress").contains("{total}"));
            assert!(locale.text("pending").contains("{n}"));
        }
    }
}
