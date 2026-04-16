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
//! An editor to modify an existing recipe.
mod actions;

use crate::cli::Edit;
use crate::config::Config;
use crate::content::RecipeItem;
use crate::mkdev_error::Error;
use crate::recipe::Recipe;

use std::collections::HashMap;

use colored::Colorize;
use inquire::error::InquireResult;
use inquire::validator::{Validation, ValueRequiredValidator};
use inquire::{Select, Text};
use rust_i18n::t;
use strum::IntoEnumIterator;

/// Invoke an editor that continuously allows inspection and editing of various data within the
/// recipe.
///
/// On exit, the user may choose to save the recipe, at which point it is canonicalised and then
/// saved to the recipe directory.
pub fn editor(args: Edit, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    // Ensure the config is loaded in memory before proceeding.
    let _config = Config::get()?;
    let mut recipe = Recipe::pick(&user_recipes, &args.recipe)?.clone();
    let mut recipe_altered = false;

    loop {
        // Target field/action,
        let action = EditorAction::select_action()?;

        // Handle quitting.
        if let EditorAction::Quit = action {
            if EditorAction::quit_menu(recipe_altered, &recipe)? {
                break;
            } else {
                continue;
            }
        }

        // Try to dispatch an edit. If it succeeds and a change is made, update `recipe_altered`
        // and print a newline for readability.
        recipe_altered |= action.dispatch(&mut recipe)?;
        println!();
    }

    Ok(())
}

/// Actions that the editor can take
#[derive(Clone, Copy, Debug, strum::EnumIter)]
enum EditorAction {
    Name,
    Description,
    AddContent,
    EditContent,
    RemoveContents,
    Quit,
}

impl EditorAction {
    /// Selects the next action the editor should take.
    fn select_action() -> InquireResult<EditorAction> {
        let message = &t!("menus.editor.what_next");
        let opts: Vec<_> = EditorAction::iter().collect();
        let vim = Config::get().unwrap().vim;

        Select::new(message, opts)
            .with_vim_mode(vim)
            .with_help_message(&t!("menus.select_help"))
            .prompt()
    }

    /// Handles exiting the editor.
    ///
    /// If the editor should close, `Ok(true)` is returned. `altered` informs whether or not a recipe has
    /// been altered. If it hasn't the editor simply closes. If the recipe has been altered, the user
    /// is presented with the choice to save and exit, exit without saving, or cancel (not exit).
    fn quit_menu(altered: bool, recipe: &Recipe) -> Result<bool, Error> {
        if !altered {
            Ok(true)
        } else {
            let opts = ExitAction::iter().collect();
            let prompt = Select::new(&t!("menus.editor.save"), opts).prompt_skippable()?;

            let Some(choice) = prompt else {
                return Ok(false);
            };

            match choice {
                ExitAction::Save => {
                    recipe.save()?;
                    Ok(true)
                }
                ExitAction::Exit => Ok(true),
                ExitAction::Cancel => Ok(false),
            }
        }
    }

    /// Inspect the field if applicable, then run the edit dialogue.
    ///
    /// Returns `Ok(true)` if an edit has been made.
    pub fn dispatch(&self, recipe: &mut Recipe) -> InquireResult<bool> {
        self.inspect(recipe);
        self.edit(recipe)
    }

    /// Dispatcher for the different fields that can be edited.
    ///
    /// Returns `Ok(true)` if an edit has been made.
    fn edit(&self, recipe: &mut Recipe) -> InquireResult<bool> {
        match self {
            Self::Name => self.edit_name(recipe),
            Self::Description => self.edit_description(recipe),
            Self::AddContent => self.add_content(recipe),
            Self::EditContent => self.edit_content(recipe),
            Self::RemoveContents => self.remove_contents(recipe),
            Self::Quit => unreachable!(),
        }
    }

    /// Present the current value of the given field if applicable, do nothing otherwise.
    fn inspect(&self, recipe: &Recipe) {
        let val = match self {
            Self::Name => format!("\"{}\"", recipe.name),
            Self::Description => format!("\"{}\"", recipe.description),
            Self::AddContent | Self::RemoveContents | Self::EditContent => recipe
                .display_contents()
                .lines()
                .collect::<Vec<_>>()
                .join("\n  "),
            _ => return,
        };

        println!(
            "{} {}:\n  {}",
            ">".green(),
            t!("menus.editor.curr_val"),
            val
        )
    }
}

/// Prompts a name for new content item.
fn prompt_new_name(current: &str, existing: &[RecipeItem]) -> InquireResult<Option<String>> {
    let msg = &t!("menus.editor.get_content_name");
    Text::new(msg)
        .with_validator(ValueRequiredValidator::new(t!(
            "menus.editor.content_name_required"
        )))
        .with_validator(|i: &str| {
            if i == current || !existing.iter().any(|item| item.name() == i) {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    t!("menus.editor.invalid_content_name").into(),
                ))
            }
        })
        .with_initial_value(current)
        .prompt_skippable()
}

impl std::fmt::Display for EditorAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AddContent => t!("editor.actions.add_content"),
                Self::Description => t!("editor.actions.description"),
                Self::EditContent => t!("editor.actions.edit_content"),
                Self::Name => t!("editor.actions.name"),
                Self::Quit => t!("general.quit"),
                Self::RemoveContents => t!("editor.actions.remove_contents"),
            }
        )
    }
}

/// Strategy enum that dictates what happens when a user tries to exit with unsaved changes.
#[derive(Clone, Copy, Debug, strum::EnumIter)]
enum ExitAction {
    Save,
    Exit,
    Cancel,
}

impl std::fmt::Display for ExitAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Save => t!("editor.exit.save"),
                Self::Exit => t!("editor.exit.exit"),
                Self::Cancel => t!("editor.exit.cancel"),
            }
        )
    }
}

#[derive(Clone, Copy, Debug, strum::EnumIter)]
enum ContentType {
    File,
    Directory,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::File => t!("general.file"),
                Self::Directory => t!("general.directory"),
            }
        )
    }
}
