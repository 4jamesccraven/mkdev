//! Interactive menus for mkdev.
mod locale;

use std::fmt::Display;

use inquire::list_option::ListOption;
use locale::*;

use crate::config::Config;
use crate::content::{RecipeItem, make_contents};
use crate::fs_wrappers::current_dir;
use crate::mkdev_error::Error;
use crate::recipe::Recipe;

use ignore::Walk;
use inquire::formatter::{BoolFormatter, MultiOptionFormatter};
use inquire::parser::BoolParser;
use inquire::{
    Confirm, MultiSelect, Text, error::InquireResult, validator::ValueRequiredValidator,
};
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

    let walk = Walk::new(cwd);
    let default_contents = make_contents(walk)?;
    recipe.contents = select_contents(default_contents)?;

    let stage = recipe.materialise(None)?;
    recipe.languages = Recipe::languages(stage.path());

    Ok(recipe)
}

/// Find out if the user really wants to delete it for real.
pub fn confirm_recipe_overwrite(message: &str, default: bool) -> InquireResult<bool> {
    let parser: BoolParser = &locale_bool_parser;
    let formatter: BoolFormatter = &locale_bool_formatter;
    let default_formatter: BoolFormatter = &locale_bool_default_formatter;

    Confirm::new(message)
        .with_default(default)
        .with_parser(parser)
        .with_formatter(formatter)
        .with_error_message(&t!("menus.invalid_yn"))
        .with_default_value_formatter(default_formatter)
        .prompt()
}

fn get_recipe_name() -> InquireResult<String> {
    Text::new(&t!("menus.get_name"))
        .with_validator(ValueRequiredValidator::new(t!("menus.name_required")))
        .prompt()
}

fn get_recipe_description() -> InquireResult<String> {
    Text::new(&t!("menus.get_desc")).prompt()
}

fn select_contents(contents: Vec<RecipeItem>) -> InquireResult<Vec<RecipeItem>> {
    let formatter: MultiOptionFormatter<RecipeItem> = &multiselect_truncate_formatter;
    let vim = Config::get()
        .expect("The config should be loaded at the top of a menu")
        .vim;

    MultiSelect::new(&t!("menus.filter_rec"), contents)
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
        0 => format!("{}", t!("menus.selected_count", count => 0)),
        1..=3 => format!(
            "{}: {}",
            t!("menus.selected_count", count => len),
            example_string
        ),
        4.. => format!(
            "{}: {}, ...",
            t!("menus.selected_count", count => len),
            example_string
        ),
    }
}
