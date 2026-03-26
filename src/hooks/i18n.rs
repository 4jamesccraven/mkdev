//! A hook that sets up internationalisation for mkdev.
use std::collections::HashSet;

use rust_i18n::{available_locales, set_locale};
use sys_locale::get_locales as preferred_locales;

pub fn hook() {
    let supported: HashSet<&str> = HashSet::from_iter(available_locales!());

    let preferred = preferred_locales()
        .map(|pref| normalise_locale(pref.as_str()))
        .find(|pref| {
            let base = pref.split(['-', '_']).next().unwrap_or(pref);
            supported.contains(pref.as_str()) || supported.contains(base)
        })
        .unwrap_or("en-US".to_string());

    set_locale(&preferred);
}

fn normalise_locale(locale: &str) -> String {
    locale
        .split_once(['-', '_'])
        .map(|(lang, region)| format!("{}-{}", lang.to_lowercase(), region.to_uppercase()))
        .unwrap_or_else(|| locale.to_lowercase())
}
