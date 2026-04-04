//! Interactive menus for mkdev.
mod locale;

use locale::*;

use crate::config::Config;
use crate::content::{RecipeItem, make_contents};
use crate::fs_wrappers::current_dir;
use crate::mkdev_error::Error;
use crate::recipe::Recipe;

use ignore::Walk;
use inquire::formatter::BoolFormatter;
use inquire::parser::BoolParser;
use inquire::{Confirm, MultiSelect, Text, error::InquireResult};
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
    Text::new(&t!("menus.get_name")).prompt()
}

fn get_recipe_description() -> InquireResult<String> {
    Text::new(&t!("menus.get_desc")).prompt()
}

fn select_contents(contents: Vec<RecipeItem>) -> InquireResult<Vec<RecipeItem>> {
    let vim = Config::get()
        .expect("The config should be loaded at the top of a menu")
        .vim;

    MultiSelect::new(&t!("menus.filter_rec"), contents)
        .with_vim_mode(vim)
        .prompt()
}
