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
//! The interface to mkdev's central recipe repository.
use super::Recipe;
use crate::config::Config;
use crate::fs_wrappers as fs;
use crate::mkdev_error::{Context, Error, Subject};
use crate::recipe::version::deserialise_recipe;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dirs::data_dir;

/// A destructured unique identifier for a recipe.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct ManifestKey {
    namespace: Option<String>,
    shortname: String,
}

impl<T: AsRef<str>> From<T> for ManifestKey {
    fn from(id: T) -> Self {
        let id_str = id.as_ref();

        match id_str.rsplit_once("::") {
            Some((namespace, shortname)) => ManifestKey {
                namespace: Some(namespace.to_string()),
                shortname: shortname.to_string(),
            },
            None => ManifestKey {
                namespace: None,
                shortname: id_str.to_string(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct StoreQuery(Vec<ManifestKey>);

impl From<ManifestKey> for StoreQuery {
    fn from(key: ManifestKey) -> Self {
        Self(vec![key])
    }
}

impl ManifestKey {
    /// Provides the full ID as a [`String`].
    pub fn to_id(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{}::{}", ns, self.shortname),
            None => self.shortname.clone(),
        }
    }
}

/// A single entry in the recipe store.
#[derive(Clone, Debug)]
pub struct ManifestEntry {
    /// The absolute path to the recipe.
    pub store_path: PathBuf,
    /// The recipe itself.
    pub recipe: Recipe,
}

type ManifestEntries = HashMap<ManifestKey, ManifestEntry>;

/// Mkdev's recipe repository.
#[derive(Clone, Debug)]
pub struct RecipeStore {
    /// The underlying mapping of recipe IDs and their location on disk.
    manifest: ManifestEntries,
}

impl RecipeStore {
    // ------------------------------------------------------------
    //  Store Initialisation
    // ------------------------------------------------------------

    /// Creates a [`RecipeStore`] from the standard store locations
    //
    // TODO: add more index locations, namely `/usr/share/mkdev`
    pub fn from_default_indices() -> Result<Self, Error> {
        let (store, _) = Self::from_indices(&[recipe_dir()?])?;
        Ok(store)
    }

    /// Creates a [`RecipeStore`] from a list of directories to index.
    ///
    /// Returns `Ok((store, invalid_recipe_paths))` if the directories can be indexed. Otherwise
    /// returns an [`Error`].
    pub fn from_indices<P>(dirs: &[P]) -> Result<(Self, Vec<PathBuf>), Error>
    where
        P: AsRef<Path>,
    {
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

        let store = Self { manifest: valid };

        Ok((store, invalid))
    }

    /// Indexes the contents of a directory as manifest entries.
    ///
    /// For each entry, if it is valid it is added to a partial manifest, and if it's not, the
    /// invalid path is stored in a list with all the others. The function itself only errs if the
    /// directory cannot be read.
    fn index_dir<P>(dir: P) -> Result<(ManifestEntries, Vec<PathBuf>), Error>
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
                                ManifestKey::from(&r.name),
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
    // Queries
    // ------------------------------------------------------------

    /// Get all information associated with a recipe ID.
    fn pick_raw(&self, recipe_id: &str) -> Result<ManifestKey, Error> {
        // Always prefer an exact match.
        let key = ManifestKey::from(recipe_id);
        if self.manifest.contains_key(&key) {
            return Ok(key);
        }

        // Failing that see if the ID is referring to the default recipe by only specifying its
        // namespace.
        let remainder_as_namespace = ManifestKey {
            namespace: Some(recipe_id.into()),
            shortname: "default".into(),
        };
        if self.manifest.contains_key(&remainder_as_namespace) {
            return Ok(remainder_as_namespace);
        }

        // As a last resort, see if the remaineder alone uniquely identifies a recipe.
        let potential_matches: Vec<_> = self
            .manifest
            .keys()
            .filter(|k| k.shortname == recipe_id)
            .collect();

        match potential_matches.len() {
            0 => Err(Error::Invalid {
                subject: Subject::Recipe,
                examples: Some(vec![recipe_id.into()]),
            }),
            1 => {
                let entry = potential_matches.into_iter().next().unwrap();
                Ok(entry.clone())
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

    /// Query the store for a single manifest entries.
    pub fn pick_one(&self, recipe_id: &str) -> Result<StoreQuery, Error> {
        self.pick_raw(recipe_id).map(StoreQuery::from)
    }

    /// Query the store for several manifest entries.
    pub fn pick_many(&self, recipe_ids: &[&str]) -> Result<StoreQuery, Error> {
        recipe_ids
            .iter()
            .map(|id| self.pick_raw(id))
            .collect::<Result<_, _>>()
            .map(StoreQuery)
    }

    /// Query the store for a namespace.
    pub fn pick_namespace(&self, namespace_id: &str) -> StoreQuery {
        let matches: Vec<_> = self
            .manifest
            .keys()
            .filter(|k| k.namespace.as_ref().is_some_and(|ns| ns == namespace_id))
            .cloned()
            .collect();

        StoreQuery(matches)
    }
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
        fs::create_dir_all(&data_dir, Context::Gather)?;
    }

    Ok(data_dir)
}
