use fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader};
use std::borrow::Cow;
use std::{collections::HashMap, sync::RwLock};
use unic_langid::langid;

fluent_templates::static_loader! {
    // Declare our `StaticLoader` named `LOCALES`.
    static LOCALES = {
        // The directory of localizations and fluent resources.
        locales: "./assets/lang",
        // The language to fallback on if something is not present.
        fallback_language: "en-US",
    };
}

static LANG: RwLock<LanguageIdentifier> = RwLock::new(langid!("en-US"));

pub struct LangDesc {
    name: &'static str,
    id: LanguageIdentifier,
}

impl LangDesc {
    const fn new(name: &'static str, id: LanguageIdentifier) -> Self {
        Self { name, id }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn id(&self) -> LanguageIdentifier {
        self.id.clone()
    }
}

pub static LANGS: [LangDesc; 8] = [
    LangDesc::new("English", langid!("en-US")),
    LangDesc::new("Deutsch", langid!("de-DE")),
    LangDesc::new("Français", langid!("fr-FR")),
    LangDesc::new("Português", langid!("pt-BR")),
    LangDesc::new("Русский", langid!("ru-RU")),
    LangDesc::new("简体中文", langid!("zh-CN")),
    LangDesc::new("日本語", langid!("ja-JP")),
    LangDesc::new("한국어", langid!("ko-KR")),
];

pub fn set_current_locale(lang_id: LanguageIdentifier) {
    *LANG.write().unwrap() = lang_id;
}

pub fn tr(text_id: &str) -> String {
    LOCALES
        .try_lookup(&LANG.read().unwrap(), text_id)
        .unwrap_or_else(|| text_id.to_string())
}

pub fn tr_a(text_id: &str, args: &[(String, FluentValue)]) -> String {
    let mut args_hm = HashMap::new();
    for (key, arg) in args.iter().cloned() {
        args_hm.insert(Cow::from(key), arg.clone());
    }
    LOCALES
        .try_lookup_with_args(&LANG.read().unwrap(), text_id, &args_hm)
        .unwrap_or_else(|| text_id.to_string())
}
