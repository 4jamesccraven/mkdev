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
use crate::fs;
use crate::mkdev_error::Context;
use crate::mkdev_error::Error;
use crate::recipe::RecipeStore;
use crate::warning;

use std::path::PathBuf;

use rust_i18n::t;

/// Deletes a recipe based on command line arguments.
pub fn delete_recipe(args: Delete, store: RecipeStore) -> Result<(), Error> {
    if args.namespace {
        return delete_namespace(&store, &args.recipe);
    }

    let query = store.pick_one(&args.recipe)?;
    // TODO: let the store handle deletion
    let to_delete = store.select(&query).next().unwrap().recipe.clone();
    let deleted_file = to_delete.delete()?;

    println!(
        "{}",
        t!("recipes.delete_msg", path => &deleted_file.display())
    );

    Ok(())
}

/// Deletes every recipe in a namespace.
fn delete_namespace(store: &RecipeStore, namespace: &str) -> Result<(), Error> {
    let query = store.pick_namespace(namespace);

    store.select(&query).try_for_each(|r| {
        let deleted_file = r.recipe.delete()?;

        println!(
            "{}",
            t!("recipes.delete_msg", path => &deleted_file.display())
        );

        Ok(())
    })
}

impl Recipe {
    /// Delete the recipe by deleting its serialised self.
    pub fn delete(&self) -> Result<PathBuf, Error> {
        let recipe_file = self.dwelling()?;

        if self.is_external()? {
            warning!("{}", t!("recipes.external"));
        }
        fs::remove_file(&recipe_file, Context::Delete)?;

        Ok(recipe_file)
    }
}
