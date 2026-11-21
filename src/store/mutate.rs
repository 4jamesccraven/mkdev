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
use crate::recipe::{Recipe, RecipeId, RecipeKind};

use std::path::{Path, PathBuf};

impl RecipeStore {
    // ------------------------------------------------------------
    //  Store Modification
    // ------------------------------------------------------------

    /// Add a recipe to the store, using the default save location.
    ///
    /// This not only adds it to the memory in-store, but saves it on disk as well.
    pub fn save(&mut self, recipe: Recipe) -> Result<PathBuf, Error> {
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
                kind: RecipeKind::default(),
            },
        );

        Ok(store_path)
    }

    /// Renames the file associated with the given recipe.
    ///
    /// `new_name` should be a **basename**, like the one supplied by [`RecipeStore::store_name`].
    ///
    /// This operation is atomic---if the path is changed, so does the entry in the store, if not it
    /// does not change.
    pub fn rename_file<K, P>(&mut self, recipe_id: K, new_name: P) -> Result<(), Error>
    where
        K: Into<RecipeId>,
        P: AsRef<Path>,
    {
        let key = self.pick_raw(&recipe_id.into())?.clone();
        let store_path = &self.manifest[&key].store_path;
        let to = store_path
            .parent()
            .expect("store paths are absolute")
            .join(new_name.as_ref());

        std::fs::rename(store_path, &to).map_err(|e| {
            // This can only happen if what's passed in is not a basename.
            //
            // see <https://doc.rust-lang.org/std/fs/fn.rename.html#errors>
            if e.kind() == std::io::ErrorKind::CrossesDevices {
                crate::borked!(e)
            }

            // This can happen if the original store path does not exist or if the user lacks
            // permissions. The former is indicative of some kind of larger corruption.
            Error::FsDenied {
                which: store_path.to_path_buf(),
                context: Context::Imprint,
            }
        })?;

        self.manifest.get_mut(&key).unwrap().store_path = to;
        Ok(())
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

    // ------------------------------------------------------------
    //  Configuration and Synchronisation
    // ------------------------------------------------------------

    /// Inserts a new index just before the default (i.e., second-most precedence when loading).
    ///
    /// This does not automatically load data from previously un-indexed directories. To load them,
    /// use [`RecipeStore::synchronise`] et al.
    pub fn push_index<P>(&mut self, index: P)
    where
        P: AsRef<Path>,
    {
        let penultimate = self.indices.len().saturating_sub(1);
        let new_index = index.as_ref().to_path_buf();
        self.indices.insert(penultimate, new_index);
    }

    /// Fully synchronise the store by reconstructing it from scratch.
    ///
    /// This function intentionally discards paths that have invalid recipe data.
    pub fn synchronise(&mut self) -> Result<(), Error> {
        self.synchronise_with_handler(|_| ())
    }

    /// Fully synchronise the store by reconstructing it from scratch.
    ///
    /// There may be some errors if this function returns [`Ok`], particularly if an indexed path
    /// does not contain valid recipe data. For each such failure, the error is propagated to the
    /// provided handler for call-site-specific error handling.
    pub fn synchronise_with_handler(&mut self, err_handler: impl Fn(Error)) -> Result<(), Error> {
        let (new_self, errs) = Self::gather_dirs(&self.indices)?;

        *self = new_self;
        errs.into_iter().for_each(err_handler);

        Ok(())
    }
}
