//! Localization: string tables in assets/data/strings_<lang>.json (see tools/strings). English is the fallback.
//! Use `strings.t("key")` / `strings.tn("key", n)`. Add a language: new strings_xx.json + register in tools/gen_data.py.
use crate::manifest::AssetManifest;
use bevy::prelude::*;
use std::collections::HashMap;

pub const LANGS: [(&str, &str); 2] = [("en", "English"), ("fi", "Suomi")];

#[derive(Resource, Default)]
pub struct Strings { pub lang: String, en: HashMap<String, String>, cur: HashMap<String, String> }

fn read(man: &AssetManifest, lang: &str) -> HashMap<String, String> {
    let id = format!("data.strings_{lang}");
    let path = format!("assets/{}", man.path(&id));
    std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

impl Strings {
    pub fn load(man: &AssetManifest, lang: &str) -> Self {
        Self { lang: lang.to_string(), en: read(man, "en"), cur: if lang == "en" { HashMap::new() } else { read(man, lang) } }
    }
    pub fn set_lang(&mut self, man: &AssetManifest, lang: &str) { *self = Self::load(man, lang); }
    pub fn t(&self, key: &str) -> String { self.cur.get(key).or_else(|| self.en.get(key)).cloned().unwrap_or_else(|| key.to_string()) }
    pub fn tn(&self, key: &str, n: impl std::fmt::Display) -> String { self.t(key).replace("{n}", &n.to_string()) }
}

pub struct I18nPlugin;
impl Plugin for I18nPlugin {
    fn build(&self, app: &mut App) {
        let man = AssetManifest::load();
        let lang = crate::settings::Settings::load().language;
        app.insert_resource(Strings::load(&man, &lang));
    }
}
