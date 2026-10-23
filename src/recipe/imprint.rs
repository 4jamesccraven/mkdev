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
//! Implementation of `mk imprint`.
//!
//! Imprinting is the intended way of making a new recipe. When a recipe is imprinted, mkdev walks
//! the current directory recursively and stores the relative path and contents of all text files
//! and subdirectories. Upon completion of this recursive walk, the contents are packed into a
//! recipe struct and stored to the recipe directory.
use super::Recipe;
use super::capture::{CaptureOpts, capture_contents_with};

use std::path::PathBuf;

use crate::cli::Imprint;
use crate::error::Context;
use crate::error::Error;
use crate::fs;
use crate::menus::{self, confirm_recipe_overwrite};
use crate::recipe::RecipeStore;
use crate::warning;

use rust_i18n::t;

/// Imprints a recipe using arguments from the command line, and post processes it accordingly.
pub fn imprint_recipe(args: Imprint, mut store: RecipeStore) -> Result<(), Error> {
    // Generate the new recipe
    let new = if args.interactive {
        menus::imprint()?
    } else {
        // Capture the current directory with filters (unless the user disables them) and with any
        // additional exclusion globs the user provides.
        let contents = capture_contents_with(CaptureOpts {
            filtered: !args.no_filter,
            overrides: args.exclude,
            ..Default::default()
        })?;

        Recipe {
            name: args.recipe,
            description: args.description.unwrap_or("".into()),
            languages: Recipe::languages("."),
            contents,
        }
    };

    // If the user wants a nix expression, we write that file and bail early.
    if let Some(path) = args.to_nix {
        return write_to_nix(path, new);
    }

    // Is the action going to overwrite an existing recipe?
    let destructive = store.recipe_exists(&new.name);

    // We can proceed if no data will be overwritten...
    let can_proceed = !destructive
        // ... or if the user overrides safety warnings.
        || args.suppress_warnings
        || (args.interactive && confirm_recipe_overwrite(&new.name)?);

    // Warn the user if the original source was a symlink instead of a file.
    //
    // This allows, e.g., home-manager users to know that the file was only changed locally instead
    // of in the source of whatever program externally manages it.
    let is_external = store.is_external(&new.name) == Some(true);
    if destructive && is_external {
        warning!("{}", t!("recipes.external"));
    }

    if !can_proceed {
        return Err(Error::DestructionWarning { name: new.name });
    }

    let save_location = store.save(new)?;
    println!("{}", save_location.display());

    Ok(())
}

/// Writes a recipe to a Nix file.
fn write_to_nix(path: PathBuf, recipe: Recipe) -> Result<(), Error> {
    let recipe_as_nix = ser_nix::to_string(&recipe)
        .expect("ser_nix's serialisation is infallible with non-path types.");

    fs::write(path, recipe_as_nix, Context::Imprint)
}
