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
//! User-facing display for recipes.
mod display_config;
mod tree;

pub use display_config::DisplayConfig;
pub use tree::repr_tree;

use crate::recipe::{Language, Recipe};
use crate::replacer::{InvalidTokenStrategy, ReplaceFmt};

use std::collections::HashMap;

use colored::Colorize;

const DELIMS: (&str, &str) = ("{", "}");
const FALLBACK: InvalidTokenStrategy = InvalidTokenStrategy::Preserve;

/// Formats and displays a list of recipes according to a provided configuration.
pub fn display_recipes_with_config(recipes: &[&Recipe], config: &DisplayConfig) -> String {
    let additional_namespaces = !recipes.iter().all(|r| r.namespace().is_none());
    let mut prev_ns = recipes.first().and_then(|r| r.namespace());

    // Only show the Global namespace header if there are others or if the user's config requests
    // it.
    let mut lines = if additional_namespaces || config.namespace_show_always {
        let first_line = cfg_display_namespace(prev_ns, config)
            .trim_start_matches('\n')
            .to_string();
        vec![first_line]
    } else {
        Vec::new()
    };

    for recipe in recipes {
        let curr_ns = recipe.namespace();
        if curr_ns != prev_ns {
            lines.push(cfg_display_namespace(curr_ns, config));
        }

        lines.push(cfg_display_recipe(recipe, config));

        prev_ns = curr_ns;
    }

    lines.join("\n") + &config.recipes_suffix
}

/// Formats a namespace divider according to a provided configuration.
fn cfg_display_namespace(namespace: Option<&str>, config: &DisplayConfig) -> String {
    let subs = HashMap::from([(
        "namespace".to_string(),
        namespace.unwrap_or("Globals").to_string(),
    )]);

    replace(subs, &config.namespace_fmt)
}

/// Formats a single recipe according to a provided configuration.
fn cfg_display_recipe(recipe: &Recipe, config: &DisplayConfig) -> String {
    let show_description = config.show_descriptions.unwrap_or(true);

    let subs = HashMap::from([
        (
            "name".to_string(),
            cfg_display_recipe_name(recipe, &config.name_fmt, config.name_bold),
        ),
        (
            "langs".to_string(),
            cfg_display_langs(
                &recipe.languages,
                &config.lang_fmt,
                &config.langs_join,
                config.lang_colour,
            ),
        ),
        (
            "desc".to_string(),
            cfg_display_description(&recipe.description, &config.desc_fmt, show_description),
        ),
    ]);

    replace(subs, &config.recipe_fmt)
}

/// Helper function to fill a format string.
#[inline(always)]
fn replace(subs: HashMap<String, String>, fmt_string: &str) -> String {
    ReplaceFmt::new(subs, DELIMS, FALLBACK).replace(fmt_string)
}

/// Displays the recipe name as configured.
fn cfg_display_recipe_name(recipe: &Recipe, fmt_string: &str, bold: bool) -> String {
    let do_cond_fmt = |name: &str| {
        if bold {
            name.bold().to_string()
        } else {
            name.to_string()
        }
    };

    let subs = HashMap::from([
        ("name".to_string(), do_cond_fmt(&recipe.name)),
        ("shortname".to_string(), do_cond_fmt(recipe.shortname())),
        (
            "namespace".to_string(),
            do_cond_fmt(recipe.namespace().unwrap_or("Global")),
        ),
    ]);
    replace(subs, fmt_string)
}

/// Displays the recipe languages as configured.
fn cfg_display_langs(
    langs: &[Language],
    fmt_string: &str,
    join_string: &str,
    show_colour: bool,
) -> String {
    langs
        .iter()
        .map(|l| {
            HashMap::from([(
                "lang".to_string(),
                if show_colour {
                    format!("{l}")
                } else {
                    l.name.clone()
                },
            )])
        })
        .map(|subs| replace(subs, fmt_string))
        .collect::<Vec<String>>()
        .join(join_string)
}

/// Displays the recipe description as configured.
fn cfg_display_description(description: &str, fmt_string: &str, show_desc: bool) -> String {
    if !show_desc {
        return "".into();
    }

    let subs = HashMap::from([("desc".to_string(), description.to_string())]);
    replace(subs, fmt_string)
}
