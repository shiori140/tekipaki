use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// 利用者が選んだ表示言語。Auto は OS の言語設定に合わせる。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguagePref {
    #[default]
    Auto,
    Ja,
    En,
}

/// 実際に表示に使う言語。画面側の辞書（i18n.js）のキーと揃える。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Ja,
    En,
}

impl LanguagePref {
    pub fn resolve(self) -> Lang {
        match self {
            LanguagePref::Ja => Lang::Ja,
            LanguagePref::En => Lang::En,
            LanguagePref::Auto => lang_from_locales(sys_locale::get_locales()),
        }
    }
}

/// 対応している言語のうち、OS の優先順で最初に見つかったものを選ぶ。どれもなければ英語。
fn lang_from_locales(locales: impl IntoIterator<Item = String>) -> Lang {
    locales
        .into_iter()
        .find_map(|tag| match tag.to_lowercase().split(['-', '_']).next() {
            Some("ja") => Some(Lang::Ja),
            Some("en") => Some(Lang::En),
            _ => None,
        })
        .unwrap_or(Lang::En)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SettingsFile {
    language: LanguagePref,
}

pub struct Settings {
    path: PathBuf,
    data: Mutex<SettingsFile>,
}

impl Settings {
    pub fn open(dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&dir)?;
        let path = dir.join("settings.json");
        // 設定は失っても困らないので、読めなければ既定値で始める。
        let data = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Ok(Settings {
            path,
            data: Mutex::new(data),
        })
    }

    pub fn language(&self) -> LanguagePref {
        self.data.lock().unwrap().language
    }

    pub fn set_language(&self, language: LanguagePref) -> std::io::Result<()> {
        let mut data = self.data.lock().unwrap();
        data.language = language;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(&*data)?)?;
        fs::rename(&tmp, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_first_supported_locale() {
        let tags = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(lang_from_locales(tags(&["ja-JP", "en-US"])), Lang::Ja);
        assert_eq!(lang_from_locales(tags(&["fr_FR", "ja"])), Lang::Ja);
        assert_eq!(lang_from_locales(tags(&["en-GB"])), Lang::En);
        assert_eq!(lang_from_locales(tags(&["fr-FR"])), Lang::En);
        assert_eq!(lang_from_locales(tags(&[])), Lang::En);
    }

    #[test]
    fn explicit_choice_ignores_os() {
        assert_eq!(LanguagePref::Ja.resolve(), Lang::Ja);
        assert_eq!(LanguagePref::En.resolve(), Lang::En);
    }

    #[test]
    fn persists_language_and_defaults_to_auto() {
        let dir = std::env::temp_dir().join(format!("tekipaki-test-{}", uuid::Uuid::new_v4()));
        let settings = Settings::open(dir.clone()).unwrap();
        assert_eq!(settings.language(), LanguagePref::Auto);
        settings.set_language(LanguagePref::En).unwrap();
        assert_eq!(Settings::open(dir.clone()).unwrap().language(), LanguagePref::En);
        fs::remove_dir_all(dir).unwrap();
    }
}
