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
use crate::cli::Imprint;
use crate::content::{build_walk, make_contents};
use crate::fs_wrappers::{self, current_dir};
use crate::menus;
use crate::mkdev_error::Context;
use crate::mkdev_error::Error::{self, *};

use std::collections::HashMap;
use std::path::PathBuf;

use ignore::Walk;
use rust_i18n::t;

/// Imprints a recipe using arguments from the command line, and post processes it accordingly.
pub fn imprint_recipe(args: Imprint, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let new = if args.interactive {
        menus::imprint()?
    } else {
        let walker = build_walk(&args)?;
        Recipe::imprint(args.recipe, args.description, walker)?
    };

    if let Some(path) = args.to_nix {
        let nix_expression = ser_nix::to_string(&new)
            .expect("ser_nix's serialisation is infallible with non-path types.");

        fs_wrappers::write(path, nix_expression, Context::Imprint)?;

        return Ok(());
    }

    // Is the action going to overwrite an existing recipe?
    let destructive = user_recipes.iter().any(|(recipe, _)| recipe == &new.name);

    // If not, proceed, otherwise we defer to the user.
    // If running interactively, use a prompt. Otherwise, check for the `-s` flag.
    let can_proceed = !destructive
        || if args.interactive {
            menus::confirm_action(&t!("menus.recipe_overwrite", recipe => &new.name), false)
                .unwrap()
        } else {
            args.suppress_warnings
        };

    if !can_proceed {
        return Err(DestructionWarning { name: new.name });
    }

    let save_location = new.save()?;

    println!("{}", &save_location.display());

    Ok(())
}

impl Recipe {
    /// Create a `Recipe` by imprinting/cloning the contents of the cwd
    pub fn imprint(name: String, description: Option<String>, walker: Walk) -> Result<Self, Error> {
        let contents = make_contents(walker, &current_dir()?)?;

        let description = description.unwrap_or("".into());

        // Converts HashMap<&name, detected_info> -> Vec<(name, num_matching_files)>
        let languages = Recipe::languages(".");

        Ok(Self {
            name,
            contents,
            languages,
            description,
        })
    }

    /// Ensures that a recipe's data is canonical.
    ///
    /// `canonicalise` builds the recipe in a temporary directory, and imprints that directory,
    /// preserving the metadata associated with the potentially non-canonical self. This ensures
    /// that all directories are explicitly modeled, for example.
    pub fn canonicalise(&self) -> Result<Self, Error> {
        let temp_dir = self.materialise(None)?;
        let contents = make_contents(Walk::new(temp_dir.path()), temp_dir.path())?;
        let languages = Recipe::languages(temp_dir.path());

        Ok(Self {
            contents,
            languages,
            ..self.clone()
        })
    }

    /// Determines if the recipe is externally managed.
    ///
    /// A recipe is considered to be externally managed if it already exists and is a symlink.
    /// If the recipe is a symlink, that implies that the source of truth for the recipe is not the
    /// file in the recipe_dir itself, and can thus be safely deleted before the recipe saves
    /// itself.
    pub fn is_external(&self) -> Result<bool, Error> {
        let recipe_file = self.dwelling()?;

        if !recipe_file.exists() {
            return Ok(false);
        }

        let metadata = std::fs::symlink_metadata(&recipe_file).map_err(|_| Error::FsDenied {
            which: recipe_file,
            context: Context::Imprint,
        })?;
        Ok(metadata.is_symlink())
    }

    /// Save the recipe object by serialising `self` into the data directory.
    pub fn save(&self) -> Result<PathBuf, Error> {
        let canonical_recipe = self.canonicalise()?;
        let recipe_file = self.dwelling()?;

        if self.is_external()? {
            std::fs::remove_file(&recipe_file).map_err(|_| FsDenied {
                which: recipe_file.clone(),
                context: Context::Imprint,
            })?;
        }

        fs_wrappers::write(
            &recipe_file,
            toml::to_string_pretty(&canonical_recipe).unwrap(),
            Context::Imprint,
        )?;

        Ok(recipe_file)
    }
}
