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
//! Implementation of `mk delete`.
//!
//! Used to delete recipes from their default location.
use super::Recipe;
use crate::cli::Delete;
use crate::mkdev_error::Context;
use crate::mkdev_error::Error;

use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use rust_i18n::t;

/// Deletes a recipe based on command line arguments.
pub fn delete_recipe(args: Delete, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let to_delete = Recipe::pick(&user_recipes, &args.recipe)?;
    let deleted_file = to_delete.delete()?;

    println!(
        "{}",
        t!("recipes.delete_msg", path => &deleted_file.display())
    );

    Ok(())
}

impl Recipe {
    /// Delete the recipe by deleting its serialised self.
    pub fn delete(&self) -> Result<PathBuf, Error> {
        let recipe_file = self.dwelling()?;

        fs::remove_file(&recipe_file).map_err(|e| match e.kind() {
            ErrorKind::PermissionDenied => Error::FsDenied {
                which: recipe_file.clone(),
                context: Context::Delete,
            },
            _ => crate::borked!(e),
        })?;

        Ok(recipe_file)
    }
}
