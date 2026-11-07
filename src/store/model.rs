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

//! [`RecipeStore`] initialisation and helper functions.
use crate::config::Config;
use crate::error::{Context, Error};
use crate::fs;
use crate::recipe::{Recipe, RecipeId};
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
    pub(super) manifest: HashMap<RecipeId, ManifestEntry>,
    /// The list of directories from which recipes are sourced.
    pub(super) indices: Vec<PathBuf>,
    /// The index in which new recipes should be stored.
    pub(super) default_dir: PathBuf,
}

impl RecipeStore {
    // ------------------------------------------------------------
    //  Store Initialisation
    // ------------------------------------------------------------

    /// Creates a [`RecipeStore`] from the standard store paths.
    ///
    /// At present, the only directory indexed is the user-configured default recipe directory. This
    /// directory, if unspecified by the user, resolves to `$XDG_DATA_HOME/mkdev`.
    ///
    /// For detailed information on the default location, see: [`dirs::data_dir`]
    //
    // TODO: add more index locations, namely `/usr/share/mkdev`
    pub fn gather() -> Result<Self, Error> {
        let (store, errs) = Self::gather_dirs(&[Self::default_index()?])?;

        // Log warnings about paths that couldn't be indexed.
        errs.into_iter().for_each(|err| {
            warning!("{err}");
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
    pub fn gather_dirs<P>(dirs: &[P]) -> Result<(Self, Vec<Error>), Error>
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

            let (v, i) = Self::gather_dir(p_ref)?;

            valid.extend(v);
            invalid.extend(i);
        }

        let indices = dirs.iter().map(|p| p.as_ref().to_path_buf()).collect();

        let store = Self {
            manifest: valid,
            indices,
            default_dir,
        };

        Ok((store, invalid))
    }

    /// Indexes the contents of a directory as manifest entries.
    ///
    /// For each entry, if it is valid it is added to a partial manifest, and if it's not, the
    /// invalid path is stored in a list with all the others. The function itself only errs if the
    /// directory cannot be read.
    fn gather_dir<P>(dir: P) -> Result<(HashMap<RecipeId, ManifestEntry>, Vec<Error>), Error>
    where
        P: AsRef<Path>,
    {
        let mut valid = HashMap::new();
        let mut invalid = Vec::new();

        // Generally speaking the only way the entry can be invalid is because the directory itself
        // can't be read or becomes unavailable, so we just map it to a generic fs denied error.
        let entries = fs::read_dir(&dir, Context::Gather)?.map(|e| {
            e.map_err(|_| Error::FsDenied {
                which: dir.as_ref().to_owned(),
                context: Context::Gather,
            })
        });

        for entry in entries {
            let entry = entry?.path();

            // If the path doesn't correspond to a file, or the file isn't a format we support (as
            // inferred by its extension), or if it has no extension, we skip it.
            let Some(ext) = get_recipe_ext(&entry) else {
                continue;
            };

            #[expect(clippy::single_match, reason = "Future formats to be added")]
            match ext {
                "toml" => {
                    let payload = fs::read_to_string(&entry, Context::Gather)?;

                    match Recipe::from_toml(&payload) {
                        Ok(r) => {
                            _ = valid.insert(
                                r.as_id(),
                                ManifestEntry {
                                    store_path: entry,
                                    recipe: r,
                                },
                            )
                        }
                        Err(e) => invalid.push(e),
                    }
                }
                _ => {}
            }
        }

        Ok((valid, invalid))
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
    pub(super) fn store_name(recipe: &Recipe) -> PathBuf {
        PathBuf::from(&recipe.name).with_extension("toml")
    }
}

/// Returns a supported recipe extension from the path if one can be found. Returns [`None`]
/// otherwise.
fn get_recipe_ext(path: &Path) -> Option<&str> {
    if !path.is_file() {
        return None;
    }

    path.extension()
        .and_then(|s| s.to_str())
        // NOTE: this uses matches because I'm adding more formats soon!
        .filter(|ext| matches!(*ext, "toml"))
}
