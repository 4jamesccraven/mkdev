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

//! The core library for mkdev.
//!
//! This module defines the Recipe Schema and its components, as well as the most fundamental
//! operations on them.
pub mod capture;
pub mod content;
mod detection;
pub mod instantiate;
pub mod language;
mod model;
mod store;
mod version;

pub use content::RecipeItem;
pub use language::Language;
pub use model::{Recipe, RecipeId};
pub use store::RecipeStore;

// TODO: remove this.
use crate::{
    error::{Context, Error},
    recipe::detection::languages_for,
};
use instantiate::{OnConflict, instantiate_contents};
use std::path::Path;
use tempfile::TempDir;

impl Recipe {
    /// Updates the recipe's language metadata to reflect its contents.
    pub fn canonicalise_languages(&mut self) {
        let new_langs = languages_for(&self.contents);
        self.languages = new_langs;
    }

    /// Creates a temporary directory with all the files in the recipe instantiated on disk.
    ///
    /// Variable substitution does not occur. This is essentially an out-of-memory representation of
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
            &self.contents,
            &instantiate::InstantiateOpts {
                root_dir: temp_dir.path().to_path_buf(),
                on_conflict: OnConflict::Overwrite,
                verbose: false,
            },
        )?;

        Ok(temp_dir)
    }

    /// Ensures that a recipe's data is canonical.
    ///
    /// `canonicalise` builds the recipe in a temporary directory, and imprints that directory,
    /// preserving the metadata associated with the potentially non-canonical self.
    pub fn to_canonical(&self) -> Result<Self, Error> {
        let temp_dir = self.materialise(None)?;
        let contents = capture::capture_contents(temp_dir.path())?;
        let languages = languages_for(&self.contents);

        Ok(Self {
            contents,
            languages,
            ..self.clone()
        })
    }
}
