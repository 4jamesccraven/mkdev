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

//! Functions that read the store's state.
//!
//! These functions go beyond simple querying and do transformations to gather raw, low-level info
//! about a user's recipes.
use super::RecipeStore;

use crate::error::{Context, Error};
use crate::fs;
use crate::recipe::{Language, RecipeId, RecipeItem, RecipeKind, version};

impl RecipeStore {
    // ------------------------------------------------------------
    //  Raw Data Accessors
    // ------------------------------------------------------------

    /// Print the recipe's contents as they appear on disk to stdout.
    pub fn recipe_cat<K>(&self, recipe_id: K) -> Result<(), Error>
    where
        K: Into<RecipeId>,
    {
        use std::io::{Write, stdout};
        let file_contents = self.recipe_data_bytes(recipe_id)?;

        stdout()
            .lock()
            .write_all(&file_contents)
            .map_err(|e| Error::PipeFailure { why: e })
    }

    /// Reads the source file for the recipe as a [`String`].
    ///
    /// This skips any compatibility conversions, giving the raw file exactly as it is on disk.
    pub fn recipe_data<K>(&self, recipe_id: K) -> Result<String, Error>
    where
        K: Into<RecipeId>,
    {
        let id = self.pick_raw(&recipe_id.into())?;
        let store_path = self.manifest[id].store_path.as_path();

        fs::read_to_string(store_path, Context::Gather)
    }

    /// Reads the source file for the recipe as bytes.
    ///
    /// This skips any compatibility conversions, giving the raw file exactly as it is on disk.
    pub fn recipe_data_bytes<K>(&self, recipe_id: K) -> Result<Vec<u8>, Error>
    where
        K: Into<RecipeId>,
    {
        let id = self.pick_raw(&recipe_id.into())?;
        let store_path = self.manifest[id].store_path.as_path();

        fs::read(store_path, Context::Gather)
    }

    // ------------------------------------------------------------
    //  Metadata Accessors
    // ------------------------------------------------------------

    /// Returns the size of the recipe file in bytes.
    ///
    /// If the recipe cannot be found or its metadata cannot be retrieved, [`None`] is returned.
    /// This method should not be used with unverified IDs.
    pub fn file_len(&self, recipe_id: &RecipeId) -> Option<u64> {
        let key = self.pick_raw(recipe_id).ok()?;
        let store_path = &self.manifest[key].store_path;

        std::fs::metadata(store_path).ok().map(|m| m.len())
    }

    /// Returns the number of files and directories (in that order) in the given recipe.
    ///
    /// Returns [`None`] if the recipe is not in the store.
    pub fn content_breakdown(&self, recipe_id: &RecipeId) -> Option<(usize, usize)> {
        let key = self.pick_raw(recipe_id).ok()?;
        let contents = &self.manifest[key].recipe.contents;

        let counts = contents
            .iter()
            .fold((0, 0), |(file_count, dir_count), item| match item {
                RecipeItem::File(_) => (file_count + 1, dir_count),
                RecipeItem::Directory(_) => (file_count, dir_count + 1),
            });

        Some(counts)
    }

    /// Returns a reference to the language metadata for the given recipe.
    ///
    /// Returns [`None`] if the recipe is not in the store.
    pub fn recipe_langs(&self, recipe_id: &RecipeId) -> Option<&[Language]> {
        let key = self.pick_raw(recipe_id).ok()?;
        let langs = self.manifest[key].recipe.languages.as_slice();

        Some(langs)
    }

    /// Attempts to determine if the recipe is externally managed (e.g., by home-manager or stow)
    /// by determining whether or not the store path is a symlink.
    ///
    /// Assuming the path exists, `true` is returned for a store path that is a symlink, otherwise
    /// `false` is returned. If the symlink metadata cannot be determined, this function's return
    /// value is inconclusive and `None` is returned.
    ///
    /// None is also returned for recipes not part of the store. Do not use unverified IDs when
    /// calling this method.
    pub fn is_external(&self, recipe_id: &RecipeId) -> Option<bool> {
        let key = self.pick_raw(recipe_id).ok()?;
        let store_path = &self.manifest[key].store_path;

        std::fs::symlink_metadata(store_path)
            .ok()
            .map(|s| s.is_symlink())
    }

    // ------------------------------------------------------------
    //  Checks
    // ------------------------------------------------------------
    /// Tests whether a recipe's filename matches the one that would be automatically assigned
    /// by the store.
    ///
    /// An [`Err`] is returned if the recipe doesn't exist in the store.
    pub fn filename_matches_id(&self, recipe_id: &RecipeId) -> Result<bool, Error> {
        let key = self.pick_raw(recipe_id)?;
        let store_path = &self.manifest[key].store_path;
        let recipe = &self.manifest[key].recipe;

        let basename = store_path.file_name();
        let expected_basename = Self::store_name(recipe);

        // This is only None if the file on disk is literally called `..`
        Ok(matches!(basename, Some(b) if b == expected_basename))
    }

    /// Determines if the recipe has the most modern format available to it by reading its metadata.
    ///
    /// An [`Err`] is returned if the recipe doesn't exist in the store.
    pub fn recipe_is_modern(&self, recipe_id: &RecipeId) -> Result<bool, Error> {
        let key = self.pick_raw(recipe_id)?;
        let version = &self.manifest[key].kind;

        let is_modern = match version {
            RecipeKind::DeclarativeSerde { version: v } if *v == version::RECIPE_VERSION_MAX => {
                true
            }
            RecipeKind::DeclarativeSerde { version: _ } => false,
        };

        Ok(is_modern)
    }
}
