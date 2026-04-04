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
use super::{Recipe, recipe_dir};
use crate::cli::Imprint;
use crate::content::{build_walk, make_contents};
use crate::fs_wrappers;
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
            menus::confirm_recipe_overwrite(
                &t!("menus.recipe_overwrite", recipe => &new.name),
                false,
            )
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
        let contents = make_contents(walker)?;

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

    /// Save the recipe object by serialising self into the data directory
    pub fn save(&self) -> Result<PathBuf, Error> {
        let mut data_dir = recipe_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs_wrappers::write(
            &data_dir,
            toml::to_string_pretty(&self).unwrap(),
            Context::Imprint,
        )?;

        Ok(data_dir)
    }
}
