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

//! The mkdev store.
//!
//! The store is a central repository that manages [`Recipe`] discovery and modification. The store
//! controls saving, deletion, and editing of recipes during the lifetime of the program, and
//! provides methods for querying its data.
//!
//! ## Example Usage
//! ```no_run
//! use mkdev::recipe::RecipeStore;
//!
//! // Standard initialisation.
//! let store = RecipeStore::from_default_indices().unwrap();
//!
//! // Get all recipes.
//! let all_recipes: Vec<_> = store.pick_all().recipes().collect();
//!
//! // Get one.
//! let my_recipe_foo = store.pick_one("foo").unwrap().recipe.clone();
//! ```
use super::{Recipe, RecipeId};

use crate::config::Config;
use crate::error::{Context, Error, Subject};
use crate::fs;
use crate::recipe::version::deserialise_recipe;
use crate::warning;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A single entry in the recipe store.
#[derive(Clone, Debug)]
pub struct ManifestEntry {
    /// The absolute path to the recipe.
    pub store_path: PathBuf,
    /// The recipe itself.
    pub recipe: Recipe,
}

/// Mkdev's recipe repository.
#[derive(Clone, Debug)]
pub struct RecipeStore {
    /// The underlying mapping of recipe IDs and their location on disk.
    manifest: HashMap<RecipeId, ManifestEntry>,
    /// The index in which new recipes should be stored.
    default_dir: PathBuf,
}

impl RecipeStore {
    // ------------------------------------------------------------
    //  Store Initialisation
    // ------------------------------------------------------------

    /// Creates a [`RecipeStore`] from the standard store paths.
    ///
    /// At present, the only directory indexed is the user-configured default recipe directory. This
    /// directory, if unspecified by the user, typically resolves to `$XDG_DATA_HOME/mkdev`.
    ///
    /// For detailed information on the default location, see: [`dirs::data_dir`]
    //
    // TODO: add more index locations, namely `/usr/share/mkdev`
    pub fn from_default_indices() -> Result<Self, Error> {
        let (store, errs) = Self::from_indices(&[Self::default_index()?])?;

        // Log warnings about paths that couldn't be indexed.
        errs.into_iter().for_each(|p| {
            warning!(
                "{}",
                Error::FsDenied {
                    which: p,
                    context: Context::Gather
                }
            );
        });

        Ok(store)
    }

    /// Creates a [`RecipeStore`] from a list of directories to index.
    ///
    /// Indices should be placed in order from least to most priority---in other words, the most
    /// important, "default" directory should come last. Each subsequent index's contents overrides
    /// the previous and so on.
    ///
    /// Returns `Ok((store, invalid_recipe_paths))` if the directories can be indexed. Otherwise
    /// returns an [`Error`]. Panics if no indices are passed.
    pub fn from_indices<P>(dirs: &[P]) -> Result<(Self, Vec<PathBuf>), Error>
    where
        P: AsRef<Path>,
    {
        let default_dir = PathBuf::from(
            dirs.iter()
                .last()
                .expect("`RecipeStore::from_indices` called with no indices.")
                .as_ref(),
        );

        let mut valid = HashMap::new();
        let mut invalid = Vec::new();
        for dir in dirs {
            let p_ref = dir.as_ref();
            if !p_ref.is_dir() {
                continue;
            }

            let (v, i) = Self::index_dir(p_ref)?;

            valid.extend(v);
            invalid.extend(i);
        }

        let store = Self {
            manifest: valid,
            default_dir,
        };

        Ok((store, invalid))
    }

    /// Indexes the contents of a directory as manifest entries.
    ///
    /// For each entry, if it is valid it is added to a partial manifest, and if it's not, the
    /// invalid path is stored in a list with all the others. The function itself only errs if the
    /// directory cannot be read.
    fn index_dir<P>(dir: P) -> Result<(HashMap<RecipeId, ManifestEntry>, Vec<PathBuf>), Error>
    where
        P: AsRef<Path>,
    {
        let mut valid = HashMap::new();
        let mut invalid = Vec::new();

        let entries = fs::read_dir(&dir, Context::Gather)?;
        for maybe_entry in entries {
            // Generally speaking the only way the entry can "fail" or be invalid is because the
            // directory itself can't be read or becomes unavailable.
            let entry = maybe_entry
                .map_err(|_| Error::FsDenied {
                    which: dir.as_ref().to_owned(),
                    context: Context::Gather,
                })?
                .path();

            if !entry.is_file() {
                continue;
            }

            // Make an assumption on how to read the recipe based on the extension.
            let ext = entry.extension().and_then(|s| s.to_str());
            // TODO: remove after adding lua support
            #[expect(clippy::single_match)]
            match ext {
                Some("toml") => {
                    let file_contents = fs::read_to_string(&entry, Context::Gather)?;

                    match deserialise_recipe(&file_contents) {
                        Some(r) => {
                            _ = valid.insert(
                                r.as_id(),
                                ManifestEntry {
                                    store_path: entry,
                                    recipe: r,
                                },
                            )
                        }
                        None => invalid.push(entry),
                    }
                }
                _ => {}
            }
        }

        Ok((valid, invalid))
    }

    // ------------------------------------------------------------
    // ID Resolution and Querying
    // ------------------------------------------------------------

    /// Resolves a potentially incomplete [`RecipeId`] against the manifest, returning a reference
    /// to the canonical key in the manifest.
    ///
    /// If no [`ManifestEntry`] matches the key, an [`Error::Invalid`] is returned. If more than one
    /// is a match, [`Error::AmbiguousShortRecipe`] is returned.
    ///
    /// ## Resolution Order
    /// The `recipe_id` argument does not have to be an exact match---the recipe store tries several
    /// strategies to match the key it receives, choosing the first to succeed:
    /// 1. An exact match always succeeds.
    /// 2. The input is interpreted as a namespace, and the default is returned if it exists (e.g.,
    ///    "foo" is interpreted as "foo::default").
    /// 3. Any recipe that's shortname matches is marked as a candidate (candidate examples for
    ///    input "foo": "python::foo", "my_stuff::foo", etc.).
    ///
    /// If the input contains both a namespace _and_ a recipe name, and the exact match fails, the
    /// other strategies are not employed, and an error is immediately returned.
    fn pick_raw(&self, recipe_id: &RecipeId) -> Result<&RecipeId, Error> {
        // Always prefer an exact match.
        if let Some((key, _)) = self.manifest.get_key_value(recipe_id) {
            return Ok(key);
        }

        // If the exact match failed with both parts specified, that recipe does not exist.
        if recipe_id.namespace.is_some() {
            return Err(Error::Invalid {
                subject: Subject::Recipe,
                examples: Some(vec![recipe_id.to_string()]),
            });
        }

        // See if the remainder part was intended as namespace:
        let remainder_as_namespace = RecipeId {
            namespace: Some(recipe_id.shortname.clone()),
            shortname: "default".into(),
        };
        if let Some((key, _)) = self.manifest.get_key_value(&remainder_as_namespace) {
            return Ok(key);
        }

        // As a last resort, see if the remainder alone uniquely identifies a recipe.
        let potential_matches: Vec<_> = self
            .manifest
            .keys()
            .filter(|k| k.shortname == recipe_id.shortname)
            .collect();

        match potential_matches.len() {
            0 => Err(Error::Invalid {
                subject: Subject::Recipe,
                examples: Some(vec![recipe_id.to_string()]),
            }),
            1 => {
                let entry = potential_matches.into_iter().next().unwrap();
                Ok(entry)
            }
            2.. => Err(Error::AmbiguousShortRecipe {
                query: recipe_id.to_string(),
                possibilities: potential_matches
                    .into_iter()
                    .map(|k| self.manifest.get(k).unwrap().recipe.name.clone())
                    .collect(),
            }),
        }
    }

    /// Query the store for a single manifest entry.
    pub fn pick_one<K>(&self, recipe_id: K) -> Result<&ManifestEntry, Error>
    where
        K: Into<RecipeId>,
    {
        let key = recipe_id.into();
        self.pick_raw(&key).map(|id| &self.manifest[id])
    }

    /// Query the store for a reference to the canonical key matching the input key.
    pub fn pick_id<K>(&self, recipe_id: K) -> Result<&RecipeId, Error>
    where
        K: Into<RecipeId>,
    {
        let recipe_id = recipe_id.into();
        self.pick_raw(&recipe_id)
    }

    /// Query the store for several manifest entries.
    pub fn pick_many<I, K>(&self, recipe_ids: I) -> Result<PickResults<'_>, Error>
    where
        I: IntoIterator<Item = K>,
        K: Into<RecipeId>,
    {
        Ok(recipe_ids
            .into_iter()
            .map(|id| id.into())
            .map(|id| self.pick_raw(&id))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|id| &self.manifest[id])
            .collect())
    }

    /// Query the store for all manifest entries.
    pub fn pick_all(&self) -> PickResults<'_> {
        self.manifest.values().collect()
    }

    /// Query the store for all manifest entries in the namespace.
    pub fn pick_namespace(&self, namespace_id: &str) -> PickResults<'_> {
        self.manifest
            .iter()
            .filter(|(k, _)| {
                k.namespace
                    .as_ref()
                    .is_some_and(|ns| ns == &namespace_id.to_owned())
            })
            .map(|(_, v)| v)
            .collect()
    }

    /// Query the store to see if an ID exists.
    pub fn recipe_exists<K>(&self, recipe_id: K) -> bool
    where
        K: Into<RecipeId>,
    {
        let key = recipe_id.into();
        self.pick_raw(&key).is_ok()
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
        let store_path = &self.manifest.get(key).unwrap().store_path;

        std::fs::symlink_metadata(store_path)
            .ok()
            .map(|s| s.is_symlink())
    }

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
            fs::create_dir_all(dir.as_ref(), Context::Gather)?;
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
        let entry_path = self
            .manifest
            .get(&query)
            .expect("manifest guaranteed to contain this id.")
            .store_path
            .clone();

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

        // Get the store paths associated with the current and updated values.
        let current_store_path = self
            .manifest
            .get(&key)
            .expect("manifest guaranteed to contain original id by this point")
            .store_path
            .clone();

        let updated_store_path = current_store_path
            .parent()
            .unwrap_or(&self.default_dir)
            .join(Self::store_name(&new_recipe));

        // If the paths are different, extra steps have to be taken to remove the old value...
        let cleanup = current_store_path != updated_store_path;

        // Save the recipe to disk and update the manifest.
        // This will always run because we gave it a default parent above.
        if let Some(dir) = updated_store_path.parent() {
            self.save_in(dir, new_recipe)?;
        }

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
    // Path Helpers
    // ------------------------------------------------------------

    /// Returns the default index for new recipes.
    fn default_index() -> Result<PathBuf, Error> {
        let data_dir = Config::get()?.recipe_dir.clone();
        fs::create_dir_all(&data_dir, Context::Gather)?;

        Ok(data_dir)
    }

    /// Determines where the default index should be if the user doesn't specify one.
    pub fn default_recipe_dir() -> PathBuf {
        dirs::data_dir()
            .expect("$HOME is not set; cannot determine data directory.")
            .join("mkdev")
    }

    /// Gives a recipe a base file name to be placed in the store.
    ///
    /// TOML is assumed, to change it use [`PathBuf::set_extension`] or similar.
    fn store_name(recipe: &Recipe) -> PathBuf {
        PathBuf::from(&recipe.name).with_extension("toml")
    }
}

/// Iterator over references to store manifest entries.
///
/// Implements convenience methods for common transformations of [`ManifestEntry`].
pub struct PickResults<'a> {
    entries: std::vec::IntoIter<&'a ManifestEntry>,
}

impl<'a> PickResults<'a> {
    /// Creates an iterator over the store paths resulting from the pick.
    pub fn paths(self) -> impl Iterator<Item = &'a Path> {
        self.map(|m| m.store_path.as_path())
    }

    /// Creates an iterator over the recipes resulting from the pick.
    pub fn recipes(self) -> impl Iterator<Item = &'a Recipe> {
        self.map(|m| &m.recipe)
    }

    /// Creates an iterator over the IDs resulting from the pick.
    pub fn ids(self) -> impl Iterator<Item = RecipeId> {
        self.map(|m| m.recipe.as_id())
    }

    /// Creates an iterator over the IDs resulting from the pick, converted to Strings.
    pub fn id_strings(self) -> impl Iterator<Item = String> {
        self.ids().map(|r| r.to_string())
    }
}

impl<'a> FromIterator<&'a ManifestEntry> for PickResults<'a> {
    fn from_iter<T: IntoIterator<Item = &'a ManifestEntry>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect::<Vec<_>>().into_iter(),
        }
    }
}

impl<'a> Iterator for PickResults<'a> {
    type Item = &'a ManifestEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }
}
