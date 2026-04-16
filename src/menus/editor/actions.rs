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
//! Implementation for editing specific attributes in the editor.
use crate::config::Config;
use crate::content::RecipeItem;
use crate::menus::multiselect_truncate_formatter;
use crate::recipe::Recipe;

use super::file_editor::FileEditor;
use super::{ContentKind, EditorAction, prompt_new_name};

use std::collections::HashSet;
use std::path::PathBuf;

use inquire::error::InquireResult;
use inquire::formatter::MultiOptionFormatter;
use inquire::validator::{Validation, ValueRequiredValidator};
use inquire::{MultiSelect, Select, Text};
use rust_i18n::t;
use strum::IntoEnumIterator;

/// Tries a skippable prompt, returning `Ok(false)` if the user skips the prompt.
macro_rules! try_prompt {
    ($expr:expr) => {
        match $expr? {
            Some(val) => val,
            None => return Ok(false),
        }
    };
}

/// Assigns a value with a new one if that new value is different.
///
/// Returns `true` if the value was changed.
#[macro_export]
macro_rules! set_if_changed {
    ($l_val:expr, $r_val:expr) => {
        if $l_val != $r_val {
            $l_val = $r_val;
            true
        } else {
            false
        }
    };
}

impl EditorAction {
    /// Change the name of the recipe.
    pub fn edit_name(&self, recipe: &mut Recipe) -> InquireResult<bool> {
        let name = try_prompt!(
            Text::new(&t!("menus.imprint.get_name"))
                .with_validator(ValueRequiredValidator::new(t!(
                    "menus.imprint.name_required"
                )))
                .with_initial_value(&recipe.name)
                .prompt_skippable()
        );

        Ok(set_if_changed!(recipe.name, name))
    }

    /// Change the description of the recipe.
    pub fn edit_description(&self, recipe: &mut Recipe) -> InquireResult<bool> {
        let desc = try_prompt!(
            Text::new(&t!("menus.imprint.get_desc"))
                .with_initial_value(&recipe.description)
                .prompt_skippable()
        );

        Ok(set_if_changed!(recipe.description, desc))
    }

    /// Add a new file or directory.
    pub fn add_content(&self, recipe: &mut Recipe) -> InquireResult<bool> {
        // Get the new content's name.
        let name = try_prompt!(
            Text::new(&t!("menus.editor.get_content_name"))
                .with_validator(ValueRequiredValidator::new(t!(
                    "menus.editor.content_name_required"
                )))
                .with_validator(|i: &str| {
                    if recipe.contents.iter().any(|item| item.name() == i) {
                        Ok(Validation::Invalid(
                            t!("menus.editor.invalid_content_name").into(),
                        ))
                    } else {
                        Ok(Validation::Valid)
                    }
                })
                .prompt_skippable()
        );
        let path = PathBuf::from(name);

        // Get the type of content being added.
        let content_kind =
            try_prompt!(Select::new(&t!(""), ContentKind::iter().collect()).prompt_skippable());

        match content_kind {
            ContentKind::File => {
                // Determine the extension so that $EDITOR can see the filetype for highlighting +
                // LSP.
                let extension = path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();

                let contents = FileEditor::new("", extension).run()?;

                // Add the file into the recipe.
                recipe.contents.push(RecipeItem::File(crate::content::File {
                    name: path,
                    content: contents,
                }));

                Ok(true)
            }
            ContentKind::Directory => {
                // Add the recipe into the directory.
                recipe.contents.push(RecipeItem::Directory(path));
                Ok(true)
            }
        }
    }

    /// Change a file or directory.
    pub fn edit_content(&self, recipe: &mut Recipe) -> InquireResult<bool> {
        let use_vim_mode: bool = Config::get().unwrap().vim;

        // Select the content to edit
        let content_item = try_prompt!(
            Select::new(&t!("menus.editor.select_content"), recipe.contents.clone())
                .with_help_message(&t!("menus.select_help"))
                .with_vim_mode(use_vim_mode)
                .prompt_skippable()
        );

        match content_item {
            RecipeItem::File(ref f) => {
                // Determine the relative path the new file.
                let name = try_prompt!(prompt_new_name(
                    f.name.to_str().unwrap_or_default(),
                    &recipe.contents
                ));
                let path = PathBuf::from(name);

                // Get the extension for LSP support.
                let extension = path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();

                // Launch the editor.
                let content = FileEditor::new(&f.content, extension).run()?;

                // Make the changes if necessary.
                Ok(recipe.replace_content(
                    &f.name,
                    RecipeItem::File(crate::content::File {
                        name: path,
                        content,
                    }),
                ))
            }
            RecipeItem::Directory(ref p) => {
                // Determine the relative path to the new directory.
                let name = try_prompt!(prompt_new_name(
                    p.to_str().unwrap_or_default(),
                    &recipe.contents
                ));

                // Make the changes if necessary.
                Ok(recipe.replace_content(p, RecipeItem::Directory(PathBuf::from(name))))
            }
        }
    }

    /// Select files and directories to remove from the recipe.
    pub fn remove_contents(&self, recipe: &mut Recipe) -> InquireResult<bool> {
        let formatter: MultiOptionFormatter<RecipeItem> = &multiselect_truncate_formatter;
        let use_vim_mode = Config::get().unwrap().vim;

        // Select contents to remove.
        let contents = try_prompt!(
            MultiSelect::new(&t!("menus.imprint.filter_rec"), recipe.contents.clone())
                .with_formatter(formatter)
                .with_help_message(&t!("menus.multiselect_help"))
                .with_vim_mode(use_vim_mode)
                .prompt_skippable()
        );

        // Bail if there's nothing to delete.
        if contents.is_empty() {
            return Ok(false);
        }

        // Remove the specified paths.
        let paths_to_remove: HashSet<_> = contents.into_iter().collect();
        recipe.contents.retain(|r| !paths_to_remove.contains(r));
        Ok(true)
    }
}
