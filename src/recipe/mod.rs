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
pub mod capture;
pub mod content;
mod delete;
mod evoke;
mod imprint;
pub mod lang;
mod list;
#[expect(dead_code)]
mod store;
mod version;

// TODO: remove this.
use store::recipe_dir;

pub use content::RecipeItem;
pub use delete::*;
pub use evoke::*;
pub use imprint::*;
pub use lang::Language;
pub use list::*;
pub use store::RecipeStore;

use crate::error::{Context, Error};
use crate::fs;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tempfile::TempDir;

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
    /// Get the recipe's short name with no namespace component.
    pub fn shortname(&self) -> &str {
        self.name
            .rsplit_once("::")
            .map(|(_, sn)| sn)
            .unwrap_or(&self.name)
    }

    /// Get the recipe's namespace.
    ///
    /// Returns `Some(namespace)` if it belongs to a named namespace, or `None` if it belongs to the
    /// global namespace.
    pub fn namespace(&self) -> Option<&str> {
        self.name.rsplit_once("::").map(|(ns, _)| ns)
    }

    /// Replaces a specific item in a recipe with a new one.
    ///
    /// Returns `true` if the value was successfully found and replaced. Returns `false` if no matching
    /// item was found or if the item was a different type (i.e., setting a File to a Directory and vice
    /// versa)
    pub fn replace_content(&mut self, old_name: &Path, new_item: RecipeItem) -> bool {
        use crate::set_if_changed;
        match self
            .contents
            .iter_mut()
            .find(|item| &item.name() == old_name)
        {
            Some(item) => match (item, new_item) {
                (RecipeItem::File(f), RecipeItem::File(other)) => {
                    set_if_changed!(f.name, other.name) || set_if_changed!(f.content, other.content)
                }
                (RecipeItem::Directory(d), RecipeItem::Directory(other)) => {
                    set_if_changed!(*d, other)
                }
                _ => false,
            },
            None => false,
        }
    }

    /// Generate a breakdown of the languages in a directory.
    pub fn languages<P>(dir: P) -> Vec<Language>
    where
        P: AsRef<Path>,
    {
        let mut breakdown: Vec<_> = hyperpolyglot::get_language_breakdown(dir)
            .iter()
            .map(|(lang, files)| (*lang, files.len()))
            .collect();

        // Sort languages by number of matching files
        breakdown.sort_by_key(|b| std::cmp::Reverse(b.1));

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
    // TODO: refactor to avoid `Option` parameter.
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

    /// The location on disk where a recipe should live.
    ///
    /// Returns `Err` if the user's data directory cannot be determined.
    pub fn dwelling(&self) -> Result<PathBuf, Error> {
        Ok(recipe_dir()?.join(format!("{}.toml", self.name)))
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
            eprintln!("{}", dest.display());
        }

        match content {
            RecipeItem::File(file) => {
                // Stop if a file would be overwritten unless the user has explicitly suppressed
                // it.
                match on_conflict {
                    OnConflict::Guard if dest.is_file() => Err(Error::DestructionWarning {
                        name: dest.to_string_lossy().into(),
                    }),
                    _ => fs::write(&dest, &file.content, Context::Evoke),
                }
            }
            RecipeItem::Directory(_) => fs::create_dir_all(&dest, Context::Evoke),
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent, Context::Evoke)?;
    }
    Ok(())
}
