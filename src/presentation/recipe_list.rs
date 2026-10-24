use super::FormatConfig;
use crate::recipe::{Language, Recipe};
use crate::replacer::{InvalidTokenStrategy, ReplaceFmt};

use std::collections::HashMap;

use colored::Colorize;

/// Formats list of recipes according to a provided configuration.
pub fn format_recipe_list(recipes: &[&Recipe], config: &FormatConfig) -> String {
    let has_custom_ns = !recipes.iter().all(|r| r.namespace().is_none());
    let mut prev_ns = recipes.first().and_then(|r| r.namespace());

    // Only show the Global namespace header if there are others or if the user's config requests
    // it.
    let mut lines = if has_custom_ns || config.namespace_show_always {
        let first_line = format_namespace(prev_ns, config)
            .trim_start_matches('\n')
            .to_string();
        vec![first_line]
    } else {
        Vec::new()
    };

    for recipe in recipes {
        let curr_ns = recipe.namespace();
        if curr_ns != prev_ns {
            lines.push(format_namespace(curr_ns, config));
        }

        lines.push(format_recipe(recipe, config));

        prev_ns = curr_ns;
    }

    lines.join(&config.recipes_join) + &config.recipes_suffix
}

/// Formats a namespace divider according to a provided configuration.
fn format_namespace(namespace: Option<&str>, config: &FormatConfig) -> String {
    let subs = HashMap::from([(
        "namespace".to_string(),
        namespace.unwrap_or("Globals").to_string(),
    )]);

    replace(subs, &config.namespace_fmt)
}

/// Formats a single recipe according to a provided configuration.
fn format_recipe(recipe: &Recipe, config: &FormatConfig) -> String {
    let show_description = config.show_descriptions.unwrap_or(true);

    let subs = HashMap::from([
        (
            "name".to_string(),
            format_recipe_name(recipe, &config.name_fmt, config.name_bold),
        ),
        (
            "desc".to_string(),
            format_description(&recipe.description, &config.desc_fmt, show_description),
        ),
        (
            "langs".to_string(),
            format_langs(
                &recipe.languages,
                &config.lang_fmt,
                &config.langs_join,
                config.lang_colour,
            ),
        ),
    ]);

    replace(subs, &config.recipe_fmt)
}

/// Displays the recipe name as configured.
fn format_recipe_name(recipe: &Recipe, fmt_string: &str, bold: bool) -> String {
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
            do_cond_fmt(recipe.namespace().unwrap_or("global")),
        ),
    ]);
    replace(subs, fmt_string)
}

/// Displays the recipe description as configured.
fn format_description(description: &str, fmt_string: &str, show_desc: bool) -> String {
    if !show_desc {
        return "".into();
    }

    let subs = HashMap::from([("desc".to_string(), description.to_string())]);
    replace(subs, fmt_string)
}

/// Displays the recipe languages as configured.
fn format_langs(
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

/// Helper function to fill a format string.
#[inline]
fn replace(subs: HashMap<String, String>, fmt_string: &str) -> String {
    let delims = ("{", "}");
    let fallback = InvalidTokenStrategy::Preserve;

    ReplaceFmt::new(subs, delims, fallback).replace(fmt_string)
}
