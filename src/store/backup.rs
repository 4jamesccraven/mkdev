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

//! Backup creation and restoration for store indices.
//!
//!
//! ## Backup Archive Format
//! Mkdev's backup format is a gzipped tarball containing recipe files, and a single json payload,
//! `mk::manifest.json`.
//!
//! Because the tarball cannot contain absolute paths, the json manifest maps each recipe's
//! absolute parent to unsigned integers composed of 32 bits (u32). These IDs are themselves used as
//! pseudo-parents inside of the tarball. The integer ID 0 denotes a recipe that has no parent,
//! something that should, in practice, be impossible.
//!
//! ### Example Manifest Mapping:
//! ```json
//! {
//!     1: "/usr/share/mkdev",
//!     2: "/home/user/.local/share/mkdev",
//!     3: "/home/user/my-special-recipes"
//! }
//! ```
//!
//! Then, individual recipes might be stored like so:
//! ```text
//! 1/shellscript.toml
//! 2/python::default.toml
//! 2/python::lib.toml
//! 2/rust.toml
//! 3/my-secret-recipe.toml
//! ```
use super::RecipeStore;

use crate::error::{Context, Error};
use crate::fs;

use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};

const MANIFEST_FILE: &str = "mk::manifest.json";

impl RecipeStore {
    // ------------------------------------------------------------
    //  Backup Creation (export)
    // ------------------------------------------------------------

    pub fn tar_indices(&self, indices: &[impl AsRef<Path>]) -> Result<Vec<u8>, Error> {
        self.tar_indices_with_progress(indices, |_, _| ())
    }

    pub fn tar_indices_with_progress(
        &self,
        indices: &[impl AsRef<Path>],
        on_progress: impl Fn(usize, &Path),
    ) -> Result<Vec<u8>, Error> {
        let existing: HashSet<_> = self.pick_indices().cloned().collect();
        let (selected, non_existent): (Vec<_>, Vec<_>) = indices
            .iter()
            .map(|p| p.as_ref())
            .partition(|&p| existing.contains(p));

        if !non_existent.is_empty() {
            let false_indices = non_existent
                .into_iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ");

            return Err(Error::Serialisation {
                // TODO: i18n
                cause: format!(
                    "attempted to serialise non-existent store indices: {}",
                    false_indices
                ),
                context: Context::Backup,
            });
        }

        let selected: HashSet<_> = selected.into_iter().collect();
        let paths: Vec<_> = self
            .pick_all()
            .paths()
            .filter_map(|p| {
                let parent = p.parent()?;
                (selected.contains(parent)).then_some(p)
            })
            .collect();

        let backup = StoreBackup::from_paths(&paths);
        self.tar_backup_with_progress(backup, on_progress)
    }

    fn tar_backup_with_progress(
        &self,
        backup: StoreBackup,
        on_progress: impl Fn(usize, &Path),
    ) -> Result<Vec<u8>, Error> {
        let mut archive = tar::Builder::new(Vec::new());

        // Serialise the manifest for the pseudo-parents as json, and write that data into the
        // tarball.
        let backup_manifest = backup.to_manifest_json()?;
        let mut header = tar::Header::new_gnu();
        header.set_size(backup_manifest.len() as u64);
        header.set_mode(0o644);
        archive
            .append_data(&mut header, MANIFEST_FILE, backup_manifest.as_bytes())
            .expect("Write on `Vec<u8>` is infallible");

        let backup_len = backup.paths.len();
        for entry in backup.paths.iter() {
            let real_path = backup
                .real_path(entry)
                .expect("this can only happen with a corrupt backup");

            archive
                .append_path_with_name(real_path, entry)
                .inspect_err(|e| eprintln!("{e}"))
                .map_err(|_| Error::FsDenied {
                    which: entry.to_path_buf(),
                    context: Context::Backup,
                })?;

            on_progress(backup_len, entry);
        }

        let tar_bytes = archive
            .into_inner()
            .expect("Write on `Vec<u8>` is infallible");

        Ok(tar_bytes)
    }

    // ---------------------------------------------------
    //  Backup Restoration (Import)
    // ---------------------------------------------------
    pub fn install_tar(&self, tar_bytes: &[u8]) -> Result<(), Error> {
        self.install_tar_with_progress(tar_bytes, |_, _| ())
    }

    pub fn install_tar_with_progress(
        &self,
        tar_bytes: &[u8],
        on_progress: impl Fn(usize, &Path),
    ) -> Result<(), Error> {
        let mut archive = tar::Archive::new(tar_bytes);
        let entries = archive
            .entries()
            .expect("infallible on newly constructed Archive object");

        let mut pseudo_paths = Vec::new();
        let mut contents = Vec::new();
        let mut manifest: Option<HashMap<u32, PathBuf>> = None;

        for entry in entries {
            let mut entry = entry.expect("read on Vec<u8> is infallible.");
            let pseudo_path = entry
                .path()
                .map_err(|e| Error::Deserialisation {
                    cause: format!("invalid path in archive: {e}"),
                    context: Context::Backup,
                })?
                .to_path_buf();

            let mut content_bytes = Vec::new();
            entry
                .read_to_end(&mut content_bytes)
                .expect("read on Vec<u8> is infallible.");

            if pseudo_path == Path::new(MANIFEST_FILE) {
                manifest = Some(serde_json::from_slice(&content_bytes).map_err(|e| {
                    Error::Deserialisation {
                        cause: e.to_string(),
                        context: Context::Backup,
                    }
                })?);
                continue;
            }

            pseudo_paths.push(pseudo_path);
            contents.push(content_bytes);
        }

        let Some(manifest) = manifest else {
            return Err(Error::Deserialisation {
                cause: "no 'mk::manifest.json' in backup".into(),
                context: Context::Backup,
            });
        };

        let real_paths = StoreBackup {
            manifest,
            paths: pseudo_paths,
        }
        .to_real_paths();

        let targets: Vec<_> = real_paths.into_iter().zip(contents).collect();
        self.install_backup_with_progress(&targets, on_progress)
    }

    pub fn install_backup_with_progress(
        &self,
        targets: &[(PathBuf, Vec<u8>)],
        on_progress: impl Fn(usize, &Path),
    ) -> Result<(), Error> {
        let target_len = targets.len();
        targets.iter().try_for_each(|(target, content)| {
            // Get the target index.
            let index = target.parent().ok_or_else(|| Error::Deserialisation {
                // TODO: i18n
                cause: format!("attempted to write orphaned recipe: {}", target.display()),
                context: Context::Backup,
            })?;

            // Ensure the index exists.
            fs::create_dir_all(index, Context::Backup)?;

            // Write the recipe.
            // TODO!!: This desyncs the store.
            // This can't be handled with `self.save_in`, either, as that could potentially change
            // data from the backup.
            fs::write(target, content, Context::Backup)?;

            on_progress(target_len, target);
            Ok(())
        })
    }
}

/// A tar friendly representation of one or more store indices.
#[derive(Debug)]
struct StoreBackup {
    /// Maps index placeholders to their absolute path.
    pub manifest: HashMap<u32, PathBuf>,
    /// Psuedo-absolute paths to recipes.
    pub paths: Vec<PathBuf>,
}

impl StoreBackup {
    /// Builds the backup's structure based on real paths on the fs.
    ///
    /// This should not be used with paths that aren't absolute.
    pub fn from_paths(paths: &[&Path]) -> Self {
        let mut dir_manifest: HashMap<PathBuf, u32> = HashMap::new();
        let mut out_paths = Vec::new();
        let mut curr_idx = 1u32;

        for &path in paths {
            let dir = path
                .parent()
                .map(PathBuf::from)
                .unwrap_or(PathBuf::from("0"));

            if !dir_manifest.contains_key(&dir) {
                dir_manifest.insert(dir.clone(), curr_idx);
                curr_idx += 1;
            }

            let manifest_parent = PathBuf::from(format!("{}", dir_manifest[&dir]));
            out_paths.push(
                manifest_parent.join(
                    path.file_name()
                        .expect("Attempted to write recipe file with no name"),
                ),
            );
        }

        let out_manifest = dir_manifest.into_iter().map(|(k, v)| (v, k)).collect();

        Self {
            manifest: out_manifest,
            paths: out_paths,
        }
    }

    /// Serialise the manifest to json.
    pub fn to_manifest_json(&self) -> Result<String, Error> {
        serde_json::to_string(&self.manifest).map_err(|e| Error::Serialisation {
            cause: e.to_string(),
            context: Context::Backup,
        })
    }

    /// Unpack the backup into a list of real paths.
    pub fn to_real_paths(&self) -> Vec<PathBuf> {
        self.paths
            .iter()
            .filter_map(|p| self.real_path(p))
            .collect()
    }

    /// Resolve a backup entry to its original filesystem.
    ///
    /// Backup entries are stored in the tar with fake parents directories, in a form like
    /// `1/my_recipe.toml`. This decodes the parent into something real like
    /// `/home/user/.local/share/mkdev/my_recipe.toml`.
    ///
    /// This function returns for `None` in the following scenarios, which are all indicative of a
    /// corrupt backup:
    /// 1. The provided path has no parent directory in the tarball
    /// 2. The provided parent directory can't be parsed as a [`u32`]
    /// 3. The provided parent directory isn't in the manifest
    /// 4. The provided path has no file name component
    pub fn real_path(&self, path: &Path) -> Option<PathBuf> {
        let parent_id = path.parent()?.to_str()?.parse::<u32>().ok()?;
        let real_parent = self.manifest.get(&parent_id)?.clone();

        path.file_name().map(|f| real_parent.join(f))
    }
}
