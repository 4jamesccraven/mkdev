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

//! Functions for determing the language metadata of a recipe.
use super::RecipeItem;

use crate::recipe::Language;

use std::collections::HashMap;

use linguist::detect_language;

/// Detects all unique languages present in the recipe's contents.
///
/// Languages are sorted descending by number of files with that detection. That is, the languages
/// that appear the most come first.
pub fn languages_for(contents: &[RecipeItem]) -> Vec<Language> {
    let language_counts =
        contents
            .iter()
            .filter_map(language_for)
            .fold(HashMap::new(), |mut map, lang| {
                let entry = map.entry(lang.name.clone()).or_insert((lang, 0u32));
                entry.1 += 1;

                map
            });

    let mut languages: Vec<_> = language_counts
        .values()
        .map(|(lang, _)| lang)
        .cloned()
        .collect();

    languages.sort_by_key(|l| std::cmp::Reverse(language_counts[&l.name].1));
    languages.dedup();

    languages
}

/// Detects the language in the [`RecipeItem`] if present.
///
/// A value is only returned if the item is a file, and if that file can be uniquely identified as
/// one language.
pub fn language_for(item: &RecipeItem) -> Option<Language> {
    let RecipeItem::File(file) = item else {
        return None;
    };

    let filename = &file.name;
    let contents = Some(file.content.as_str());

    let detections = detect_language(filename, contents).ok()?;

    if let Some(lang) = detections.first()
        && detections.len() == 1
    {
        Some(Language {
            name: lang.name.to_string(),
            colour: lang.definition.color.as_deref().and_then(parse_hex_colour),
        })
    } else {
        None
    }
}

/// Parse a hexadecimal string with a leading '#' into a tuple of [`u8`]s
pub fn parse_hex_colour<S>(colour_str: S) -> Option<(u8, u8, u8)>
where
    S: AsRef<str>,
{
    let s = colour_str.as_ref().strip_prefix('#')?;
    Some((
        u8::from_str_radix(&s[0..2], 16).ok()?,
        u8::from_str_radix(&s[2..4], 16).ok()?,
        u8::from_str_radix(&s[4..6], 16).ok()?,
    ))
}
