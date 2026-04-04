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
//! mkdev's core library. Defines the recipe schema and provides tools for working with them.
mod delete;
mod evoke;
mod imprint;
mod lang;
mod list;
mod version;

pub use delete::*;
pub use evoke::*;
pub use imprint::*;
pub use lang::Language;
pub use list::*;

use tempfile::TempDir;
use version::*;

use crate::config::Config;
use crate::content::RecipeItem;
use crate::fs_wrappers;
use crate::mkdev_error::{Context, Error};
use crate::warning;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dirs::data_dir;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

/// A mkdev recipe (v2).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Recipe {
    /// A unique identifier for the recipe.
    ///
    /// Determines both the name of the file the recipe is stored in as well as how mkdev will
    /// display information about it.
    pub name: String,
    /// A description of the recipe (Optional)
    #[serde(default = "String::new")]
    pub description: String,
    /// The programming languages (or file formats) found in the recipe's contents.
    pub languages: Vec<Language>,
    /// The contents the recipe holds.
    pub contents: Vec<RecipeItem>,
}

impl Recipe {
    /// Gathers all recipes from the user directory.
    ///
    /// Only files with the .toml extension are checked. An invalid recipe gives a warning.
    pub fn gather() -> Result<HashMap<String, Recipe>, Error> {
        let data_dir = recipe_dir()?;
        let files = fs_wrappers::read_dir(data_dir, Context::Gather)?;
        let mut recipes: Vec<Recipe> = Vec::new();

        for file in files {
            let path = file.map_err(|e| crate::borked!(e)).unwrap().path();

            if path.extension() == Some(std::ffi::OsStr::new("toml")) && path.is_file() {
                let file_contents = fs_wrappers::read_to_string(&path, Context::Gather)?;
                let recipe = deserialise_recipe(&file_contents);

                match recipe {
                    Some(recipe) => {
                        recipes.push(recipe);
                    }
                    None => {
                        warning!("{}", t!("warnings.invalid_recipe", path => path.display()));
                    }
                }
            }
        }

        let recipes = recipes
            .iter()
            .map(|r| (r.name.clone(), r.to_owned()))
            .collect();

        Ok(recipes)
    }

    pub fn languages<P>(dir: P) -> Vec<Language>
    where
        P: AsRef<Path>,
    {
        let mut breakdown: Vec<_> = hyperpolyglot::get_language_breakdown(dir)
            .iter()
            .map(|(lang, files)| (*lang, files.len()))
            .collect();

        // Sort languages by number of matching files
        breakdown.sort_by(|a, b| b.1.cmp(&a.1));

        breakdown
            .iter()
            // Discard the count, as we only needed it to sort
            .map(|(lang, _)| {
                hyperpolyglot::Language::try_from(*lang)
                    .expect("detected language come pre-validated.")
            })
            .map(Language::from)
            .collect()
    }

    /// Creates a temporary directory with all the files in the recipe instantiated on disk.
    ///
    /// Variable substitution does not occur. This is esentially an out-of-memory representation of
    /// the recipe's `contents` field.
    pub fn materialise(&self, maybe_dir: Option<&Path>) -> Result<TempDir, Error> {
        let maybe_temp = match maybe_dir {
            Some(ref dir) => tempfile::tempdir_in(dir),
            None => tempfile::tempdir(),
        };

        let temp_dir = maybe_temp.map_err(|_| Error::FsDenied {
            which: maybe_dir
                .map(|p| p.into())
                .unwrap_or_else(std::env::temp_dir),
            context: Context::Tempfile,
        })?;

        instantiate_contents(
            temp_dir.path(),
            &self.contents,
            OnConflict::Overwrite,
            false,
        )?;

        Ok(temp_dir)
    }
}

/// Builds a single recipe by taking in its contents and writing them to disk.
///
/// If a file in the target directory already exists and the conflict resolution strategy is not
/// `Overwrite`, a Destruction error is returned.
pub fn instantiate_contents(
    dir: &Path,
    contents: &[RecipeItem],
    on_conflict: OnConflict,
    verbose: bool,
) -> Result<(), Error> {
    contents.iter().try_for_each(|content| {
        let dest = dir.join(content.name());
        ensure_parent(&dest)?;

        if verbose {
            eprintln!("{}", &dest.display());
        }

        match content {
            RecipeItem::File(file) => {
                // Stop if a file would be overwritten unless the user has explicitly suppressed
                // it.
                match on_conflict {
                    OnConflict::Guard if dest.is_file() => Err(Error::DestructionWarning {
                        name: dest.to_string_lossy().into(),
                    }),
                    _ => fs_wrappers::write(&dest, &file.content, Context::Evoke),
                }
            }
            RecipeItem::Directory(_) => fs_wrappers::create_dir_all(&dest, Context::Evoke),
        }
    })
}

/// What should happen if a file already exists in the target directory during evocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnConflict {
    Overwrite,
    Guard,
}

/// Ensures that all parent directories of a file exist.
fn ensure_parent(path: &Path) -> Result<(), Error> {
    let parent = match path.parent() {
        Some(p) => p,
        None => return Ok(()),
    };

    if !parent.is_dir() {
        fs_wrappers::create_dir_all(parent, Context::Evoke)?;
    }

    Ok(())
}

/// Gets the user's preferred data dir, or uses the default XDG_DATA_DIR.
pub fn recipe_dir() -> Result<PathBuf, Error> {
    let cfg = Config::get()?;

    let data_dir = match &cfg.recipe_dir {
        Some(dir) => dir.clone(),
        None => {
            let mut temp = data_dir().expect("$HOME is not set; cannot determine data directory.");
            temp.push("mkdev");
            temp
        }
    };

    if !data_dir.is_dir() {
        fs_wrappers::create_dir_all(&data_dir, Context::Gather)?;
    }

    Ok(data_dir)
}
