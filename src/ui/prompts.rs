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

//! Generic prompts to power interactive workflows.
use super::locale::*;

// Docs-only
#[expect(unused)]
use crate::recipe::Recipe;

use crate::config::Config;
use crate::recipe::RecipeItem;

use std::fmt::Display;

use inquire::error::InquireResult;
use inquire::formatter::{BoolFormatter, MultiOptionFormatter};
use inquire::list_option::ListOption;
use inquire::parser::BoolParser;
use inquire::validator::ValueRequiredValidator;
use inquire::{Confirm, MultiSelect, Text};
use rust_i18n::t;

/// Prompt the user for a confirmation. The message informs the user what is being confirmed.
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

/// Prompt the user to determine if a recipe can be overwritten.
pub fn confirm_recipe_overwrite(recipe_name: &str) -> InquireResult<bool> {
    confirm_action(&t!("menus.recipe_overwrite", recipe => recipe_name), false)
}

/// Prompt a recipe name.
///
/// Ensures the name is non-empty.
pub(super) fn get_recipe_name() -> InquireResult<String> {
    Text::new(&t!("menus.imprint.get_name"))
        .with_validator(ValueRequiredValidator::new(t!(
            "menus.imprint.name_required"
        )))
        .prompt()
}

/// Prompt a recipe description.
pub(super) fn get_recipe_description() -> InquireResult<String> {
    Text::new(&t!("menus.imprint.get_desc")).prompt()
}

/// Have the user select one or more [`Recipe`]s by their ID.
///
/// The IDs are accepted and returned as [`String`] for simplicity
pub fn select_recipes(recipes: Vec<String>) -> InquireResult<Vec<String>> {
    let vim = Config::get().unwrap().vim;

    MultiSelect::new(&t!("menus.evoke.pick"), recipes)
        .with_help_message(&t!("menus.multiselect_help"))
        .with_vim_mode(vim)
        .prompt()
}

/// Have the user select one or more [`RecipeItem`]s.
pub(super) fn select_contents(contents: Vec<RecipeItem>) -> InquireResult<Vec<RecipeItem>> {
    let formatter: MultiOptionFormatter<RecipeItem> = &multiselect_truncate_formatter;
    let vim = Config::get().unwrap().vim;

    MultiSelect::new(&t!("menus.imprint.filter_rec"), contents)
        .with_all_selected_by_default()
        .with_formatter(formatter)
        .with_help_message(&t!("menus.multiselect_help"))
        .with_vim_mode(vim)
        .prompt()
}

/// A truncating formatter for [`inquire::MultiSelect`].
///
/// Takes at most three items from the list and displays them comma-separated. If more than three
/// are present, `"..."` is displayed as the "fourth element", and the rest are hidden.
pub(super) fn multiselect_truncate_formatter<T>(opts: &[ListOption<&T>]) -> String
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
