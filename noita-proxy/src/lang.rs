use std::{collections::HashMap, sync::RwLock};

use fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader};
use unic_langid::langid;

fluent_templates::static_loader! {
    // Declare our `StaticLoader` named `LOCALES`.
    static LOCALES = {
        // The directory of localisations and fluent resources.
        locales: "./assets/lang",
        // The language to falback on if something is not present.
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

pub static LANGS: [LangDesc; 2] = [
    LangDesc::new("English", langid!("en-US")),
    LangDesc::new("Русский", langid!("ru-RU")),
];

pub fn set_current_locale(lang_id: LanguageIdentifier) {
    *LANG.write().unwrap() = lang_id;
}

pub fn tr(text_id: &str) -> String {
    LOCALES
        .try_lookup(&LANG.read().unwrap(), text_id)
        .unwrap_or_else(|| text_id.to_string())
}

pub fn tr_a(text_id: &str, args: &[(&str, FluentValue)]) -> String {
    let mut args_hm = HashMap::new();
    for (key, arg) in args.iter() {
        args_hm.insert(key, arg.clone());
    }
    LOCALES
        .try_lookup_with_args(&LANG.read().unwrap(), text_id, &args_hm)
        .unwrap_or_else(|| text_id.to_string())
}
