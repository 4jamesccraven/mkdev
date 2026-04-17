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
//! Interactive menus for mkdev.
mod editor;
mod locale;

use std::collections::HashMap;
use std::fmt::Display;

pub use editor::editor;
use locale::*;

use crate::config::Config;
use crate::content::{RecipeItem, make_contents};
use crate::fs_wrappers::current_dir;
use crate::mkdev_error::Error;
use crate::recipe::Recipe;

use ignore::Walk;
use inquire::error::InquireResult;
use inquire::formatter::{BoolFormatter, MultiOptionFormatter};
use inquire::list_option::ListOption;
use inquire::parser::BoolParser;
use inquire::validator::ValueRequiredValidator;
use inquire::{Confirm, MultiSelect, Text};
use rust_i18n::t;

/// Interactively imprint a recipe from the current working directory.
pub fn imprint() -> Result<Recipe, Error> {
    // Ensure the config is loaded in memory before proceeding.
    let _config = Config::get()?;
    let mut recipe = Recipe::default();
    let cwd = current_dir()?;

    let name = get_recipe_name()?;
    recipe.name = name;

    let description = get_recipe_description()?;
    recipe.description = description;

    let walk = Walk::new(&cwd);
    let default_contents = make_contents(walk, &cwd)?;
    recipe.contents = select_contents(default_contents)?;

    Ok(recipe)
}

/// Interactively select recipes to evoke.
pub fn evoke(recipes: &HashMap<String, Recipe>) -> Result<Vec<&Recipe>, Error> {
    let _config = Config::get()?;
    let selection = select_recipes(recipes.keys().collect())?;

    Ok(selection
        .into_iter()
        .map(|k| recipes.get(&k).unwrap())
        .collect())
}

/// Prompt the user to confirm something.
pub fn confirm_action(message: &str, default: bool) -> InquireResult<bool> {
    let parser: BoolParser = &locale_bool_parser;
    let formatter: BoolFormatter = &locale_bool_formatter;
    let default_formatter: BoolFormatter = &locale_bool_default_formatter;

    Confirm::new(message)
        .with_default(default)
        .with_parser(parser)
        .with_formatter(formatter)
        .with_error_message(&t!("menus.imprint.invalid_yn"))
        .with_default_value_formatter(default_formatter)
        .prompt()
}

fn get_recipe_name() -> InquireResult<String> {
    Text::new(&t!("menus.imprint.get_name"))
        .with_validator(ValueRequiredValidator::new(t!(
            "menus.imprint.name_required"
        )))
        .prompt()
}

fn get_recipe_description() -> InquireResult<String> {
    Text::new(&t!("menus.imprint.get_desc")).prompt()
}

fn select_recipes(recipes: Vec<&String>) -> InquireResult<Vec<String>> {
    let vim = Config::get().unwrap().vim;

    MultiSelect::new(&t!("menus.evoke.pick"), recipes)
        .with_help_message(&t!("menus.multiselect_help"))
        .with_vim_mode(vim)
        .prompt()
        .map(|xs| xs.iter().map(|x| x.to_string()).collect())
}

fn select_contents(contents: Vec<RecipeItem>) -> InquireResult<Vec<RecipeItem>> {
    let formatter: MultiOptionFormatter<RecipeItem> = &multiselect_truncate_formatter;
    let vim = Config::get().unwrap().vim;

    MultiSelect::new(&t!("menus.imprint.filter_rec"), contents)
        .with_all_selected_by_default()
        .with_formatter(formatter)
        .with_help_message(&t!("menus.multiselect_help"))
        .with_vim_mode(vim)
        .prompt()
}

fn multiselect_truncate_formatter<T>(opts: &[ListOption<&T>]) -> String
where
    T: Display,
{
    let len = opts.len();
    let examples: Vec<_> = opts[0..len.min(3)].iter().map(|s| s.to_string()).collect();
    let example_string = examples.join(", ");

    match len {
        0 => format!("{}", t!("menus.imprint.selected_count", count => 0)),
        1..=3 => format!(
            "{}: {}",
            t!("menus.imprint.selected_count", count => len),
            example_string
        ),
        4.. => format!(
            "{}: {}, ...",
            t!("menus.imprint.selected_count", count => len),
            example_string
        ),
    }
}
