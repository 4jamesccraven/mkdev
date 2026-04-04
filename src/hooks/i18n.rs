// mkdev - Save your boilerplate instead of writing it
// Copyright (C) 2026  James C. Craven <4jamesccraven@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
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
