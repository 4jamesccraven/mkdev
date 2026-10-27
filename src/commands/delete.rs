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
use super::StoreCommand;

use crate::cli::completion::recipe_completer;
use crate::error::Error;
use crate::recipe::RecipeStore;
use crate::warn_if_external;

use clap::Parser;
use clap_complete::ArgValueCompleter;
use rust_i18n::t;

#[derive(Parser, Debug)]
pub struct Delete {
    /// The recipe/namespace to delete
    #[arg(add = ArgValueCompleter::new(recipe_completer))]
    pub recipe: String,

    /// Delete an entire namespace.
    #[arg(long)]
    pub namespace: bool,
}

impl StoreCommand for Delete {
    fn execute(self, store: RecipeStore) -> Result<(), Error> {
        delete_command(self, store)
    }
}

/// Deletes a recipe based on command line arguments.
pub fn delete_command(args: Delete, mut store: RecipeStore) -> Result<(), Error> {
    // Handle namespace deletion
    if args.namespace {
        return delete_namespace(&mut store, &args.recipe);
    }

    // Warn the user if the store_path is a symlink.
    let id = store.pick_id(&args.recipe)?.clone();
    warn_if_external!(&id, store);

    // Delete and confirm the path deleted.
    let deleted_file = store.delete(id)?;
    println!(
        "{}",
        t!("recipes.delete_msg", path => &deleted_file.display())
    );

    Ok(())
}

/// Deletes every recipe in a namespace.
fn delete_namespace(store: &mut RecipeStore, namespace: &str) -> Result<(), Error> {
    let recipe_ids: Vec<_> = store.pick_namespace(namespace).ids().collect();

    recipe_ids.into_iter().try_for_each(|id| {
        warn_if_external!(&id, store);

        let deleted_file = store.delete(id)?;
        println!(
            "{}",
            t!("recipes.delete_msg", path => &deleted_file.display())
        );
        Ok(())
    })
}
