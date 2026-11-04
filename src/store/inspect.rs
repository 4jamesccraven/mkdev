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
use crate::recipe::RecipeId;

impl RecipeStore {
    /// Print the recipe's contents as they appear on disk to stdout.
    pub fn recipe_cat<K>(&self, recipe_id: K) -> Result<(), Error>
    where
        K: Into<RecipeId>,
    {
        use std::io::{Write, stdout};
        let file_contents = self.recipe_data_bytes(recipe_id)?;

        // If we can't even print what's the point?
        // We don't particularly about broken pipe errors, either.
        _ = stdout().lock().write_all(&file_contents);

        Ok(())
    }

    /// Reads the source file for the recipe as a [`String`].
    ///
    /// This skips any compatibility conversions, giving the raw file exactly as it is on disk.
    pub fn recipe_data<K>(&self, recipe_id: K) -> Result<String, Error>
    where
        K: Into<RecipeId>,
    {
        let id = self.pick_raw(&recipe_id.into())?;
        let store_path = self.manifest[&id].store_path.as_path();

        Ok(fs::read_to_string(store_path, Context::Gather)?)
    }

    /// Reads the source file for the recipe as bytes.
    ///
    /// This skips any compatibility conversions, giving the raw file exactly as it is on disk.
    pub fn recipe_data_bytes<K>(&self, recipe_id: K) -> Result<Vec<u8>, Error>
    where
        K: Into<RecipeId>,
    {
        let id = self.pick_raw(&recipe_id.into())?;
        let store_path = self.manifest[&id].store_path.as_path();

        Ok(fs::read(store_path, Context::Gather)?)
    }
}
