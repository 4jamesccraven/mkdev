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

//! Implementation of `mk alter`.
use crate::cli::Alter;
use crate::config::Config;
use crate::error::Error;
use crate::recipe::RecipeStore;
use crate::ui::run_editor;
use crate::warning;

use rust_i18n::t;

pub fn alter_recipe(args: Alter, mut store: RecipeStore) -> Result<(), Error> {
    // Ensure the config is loaded in memory before proceeding.
    // TODO: see about getting `config.vim` here instead.
    _ = Config::get()?;

    let original_recipe = store.pick_one(&args.recipe)?.recipe.clone();
    let original_is_external = store.is_external(&original_recipe.as_id()) == Some(true);

    if let Some(new_recipe) = run_editor(&original_recipe)? {
        // Issue a warning for changes to external data.
        if store.is_external(&new_recipe.as_id()) == Some(true) || original_is_external {
            warning!("{}", t!("recipes.external"));
        }

        // TODO: this should be the store's responsibility
        let new_recipe = new_recipe.to_canonical()?;

        // Store the recipe's new path.
        store.replace(&original_recipe, new_recipe)?;
    }

    Ok(())
}
