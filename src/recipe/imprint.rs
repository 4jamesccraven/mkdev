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
use super::{
    Recipe,
    capture::{build_walk, make_contents},
};

use crate::cli::Imprint;
use crate::error::Context;
use crate::error::Error::{self, *};
use crate::fs::{self, current_dir};
use crate::recipe::RecipeStore;
use crate::{menus, warning};

use ignore::Walk;
use rust_i18n::t;

/// Imprints a recipe using arguments from the command line, and post processes it accordingly.
pub fn imprint_recipe(args: Imprint, mut store: RecipeStore) -> Result<(), Error> {
    let new = if args.interactive {
        menus::imprint()?
    } else {
        let walker = build_walk(&args)?;
        Recipe::imprint(args.recipe, args.description, walker)?
    };

    if let Some(path) = args.to_nix {
        let nix_expression = ser_nix::to_string(&new)
            .expect("ser_nix's serialisation is infallible with non-path types.");

        fs::write(path, nix_expression, Context::Imprint)?;

        return Ok(());
    }

    // Is the action going to overwrite an existing recipe?
    let destructive = store.recipe_exists(&new.name);
    let is_external = store.is_external(&new.name) == Some(true);

    // If not, proceed, otherwise we defer to the user.
    // If running interactively, use a prompt. Otherwise, check for the `-s` flag.
    let can_proceed = !destructive
        || if args.interactive {
            menus::confirm_action(&t!("menus.recipe_overwrite", recipe => &new.name), false)
                .unwrap()
        } else {
            args.suppress_warnings
        };

    if destructive && is_external {
        warning!("{}", t!("recipes.external"));
    }

    if !can_proceed {
        return Err(DestructionWarning { name: new.name });
    }

    let save_location = store.save(new)?;

    println!("{}", save_location.display());

    Ok(())
}

impl Recipe {
    /// Create a `Recipe` by imprinting/cloning the contents of the cwd
    pub fn imprint(name: String, description: Option<String>, walker: Walk) -> Result<Self, Error> {
        let description = description.unwrap_or("".into());
        let languages = Recipe::languages(".");
        let contents = make_contents(walker, &current_dir()?)?;

        Ok(Self {
            name,
            description,
            languages,
            contents,
        })
    }

    /// Ensures that a recipe's data is canonical.
    ///
    /// `canonicalise` builds the recipe in a temporary directory, and imprints that directory,
    /// preserving the metadata associated with the potentially non-canonical self.
    #[expect(unused)]
    pub fn to_canonical(&self) -> Result<Self, Error> {
        let temp_dir = self.materialise(None)?;
        let contents = make_contents(Walk::new(temp_dir.path()), temp_dir.path())?;
        let languages = Recipe::languages(temp_dir.path());

        Ok(Self {
            contents,
            languages,
            ..self.clone()
        })
    }
}
