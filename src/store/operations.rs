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

//! Functions that mutate the store's state.
use super::model::{ManifestEntry, RecipeStore};

use crate::error::{Context, Error};
use crate::fs;
use crate::recipe::{Recipe, RecipeId};

use std::path::{Path, PathBuf};

impl RecipeStore {
    // ------------------------------------------------------------
    // Store Modification
    // ------------------------------------------------------------

    /// Add a recipe to the store, using the default save location.
    ///
    /// This not only adds it to the memory in-store, but saves it on disk as well.
    pub fn save(&mut self, recipe: Recipe) -> Result<PathBuf, Error> {
        // This seems to be unavoidable without unsafe. It sucks because it's a waste,
        // but I think it's worth it to have the ability to specify the directory without
        // duplicating logic. A single clone is worth potential errors and inconsistencies later.
        // TODO: see if this clone can be avoided.
        let dir = self.default_dir.clone();
        self.save_in(dir, recipe)
    }

    /// Add a recipe to the store.
    ///
    /// This not only adds it to the memory in-store, but saves it on disk as well.
    pub fn save_in<P>(&mut self, dir: P, recipe: Recipe) -> Result<PathBuf, Error>
    where
        P: AsRef<Path>,
    {
        // Ensure that the intended directory exists. Unless it's the default index, which is
        // verified once at init time.
        if dir.as_ref() != self.default_dir {
            fs::create_dir_all(dir.as_ref(), Context::Imprint)?;
        }

        // Calculate the store path.
        let basename = Self::store_name(&recipe);
        let store_path = dir.as_ref().join(basename);

        // Serialise the recipe and store it on disk.
        let contents = toml::to_string_pretty(&recipe)
            .expect("Recipes are generated at runtime in a toml-serialisable way.");
        fs::write(&store_path, contents, Context::Imprint)?;

        // Add to manifest.
        let recipe_key = recipe.as_id();
        self.manifest.insert(
            recipe_key,
            ManifestEntry {
                store_path: store_path.clone(),
                recipe,
            },
        );

        Ok(store_path)
    }

    /// Delete the recipe from the store.
    ///
    /// This operation is atomic---if the recipe is successfully removed from disk, it is removed
    /// from the manifest. If it can't be removed from disk it remains in the manifest.
    pub fn delete<K>(&mut self, recipe_id: K) -> Result<PathBuf, Error>
    where
        K: Into<RecipeId>,
    {
        let key = recipe_id.into();
        // Query the manifest for the given entry.
        let query = self.pick_raw(&key)?.clone();

        // Convert that ID into a store_path to remove.
        let entry_path = self.manifest[&query].store_path.clone();

        // Delete the corresponding store path.
        fs::remove_file(&entry_path, Context::Delete)?;

        // Remove from the manifest.
        self.manifest
            .remove(&query)
            .expect("manifest guaranteed to contain this id.");
        Ok(entry_path)
    }

    /// Replace the recipe at `recipe_id` with a different one.
    ///
    /// After this method is called, it is **not** guaranteed that the manifest contains the
    /// original `recipe_id`, as the new_recipe may have a different derived ID, especially when it
    /// has a different name than the original.
    pub fn replace<K>(&mut self, recipe_id: K, new_recipe: Recipe) -> Result<(), Error>
    where
        K: Into<RecipeId>,
    {
        // Verify that the ID exists in the first place.
        let key = self.pick_raw(&recipe_id.into())?.clone();

        // Canonicalise the new recipe
        let new_recipe = new_recipe.into_canonical();

        // Get the store paths associated with the current and updated values.
        let current_store_path = self.manifest[&key].store_path.clone();

        let updated_store_path = current_store_path
            .parent()
            .unwrap_or(&self.default_dir)
            .join(Self::store_name(&new_recipe));

        // If the paths are different, extra steps have to be taken to remove the old value...
        let cleanup = current_store_path != updated_store_path;

        // Save the recipe to disk and update the manifest.
        let dir = updated_store_path.parent().expect("default provided above");
        self.save_in(dir, new_recipe)?;

        // If there was a different starting file, we remove it.
        if cleanup {
            fs::remove_file(&current_store_path, Context::Imprint)?;
            // If the paths were different, so was the ID. The stale ID must be removed.
            _ = self
                .manifest
                .remove(&key)
                .expect("`RecipeStore::replace`: original recipe ID could not be removed");
        }

        Ok(())
    }
}
