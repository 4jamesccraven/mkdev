mod display_config;

pub use display_config::DisplayConfig;

use crate::recipe::{Language, Recipe};
use crate::replacer::{ReplaceFmt, UnknownToken};

use std::collections::HashMap;

use colored::Colorize;

const DELIMS: (&str, &str) = ("{", "}");
const FALLBACK: UnknownToken = UnknownToken::Preserve;

pub fn display_recipes_with_config(recipes: &[&Recipe], config: &DisplayConfig) -> String {
    recipes
        .iter()
        .map(|r| cfg_display_recipe(r, config))
        .collect::<Vec<String>>()
        .join(&config.recipes_join)
}

fn cfg_display_recipe(recipe: &Recipe, config: &DisplayConfig) -> String {
    let show_description = config.show_descriptions.unwrap_or(true);

    let subs = HashMap::from([
        (
            "name".to_string(),
            cfg_display_recipe_name(&recipe.name, &config.name_fmt, config.name_bold),
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

fn replace(subs: HashMap<String, String>, fmt_string: &str) -> String {
    ReplaceFmt::new(subs, DELIMS, FALLBACK).replace(fmt_string)
}

fn cfg_display_recipe_name(name: &str, fmt_string: &str, bold: bool) -> String {
    let name_fmt = if bold {
        name.to_string().bold().to_string()
    } else {
        name.to_string()
    };

    let subs = HashMap::from([("name".to_string(), name_fmt)]);
    replace(subs, fmt_string)
}

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

fn cfg_display_description(description: &str, fmt_string: &str, show_desc: bool) -> String {
    if !show_desc {
        return "".into();
    }

    let subs = HashMap::from([("desc".to_string(), description.to_string())]);
    replace(subs, fmt_string)
}
