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

//! Implementation of `mk store backup`.
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
use super::*;

use crate::cli::completion::index_completer;
use crate::error::{Context, Subject};
use crate::fs;

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap_complete::ArgValueCompleter;
use flate2::Compression;
use flate2::write::GzEncoder;

#[derive(Parser, Debug)]
pub struct Backup {
    /// Specific index to back up (optional).
    #[arg(add = ArgValueCompleter::new(index_completer))]
    pub index: Option<PathBuf>,

    /// Whether to show a progress bar (useful for large backups).
    #[arg(short, long)]
    pub progress: bool,

    /// Where to store the backup.
    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,
}

impl MkdevCommand for Backup {
    fn execute(self, store: RecipeStore) -> Result<(), Error> {
        backup_command(self, store)
    }
}

pub fn backup_command(args: Backup, store: RecipeStore) -> Result<(), Error> {
    // Insure valid index if one is specified.
    if let Some(ref idx) = args.index
        && !store.pick_indices().any(|i| i == idx)
    {
        return Err(Error::Invalid {
            subject: Subject::Index,
            examples: Some(vec![format!("{}", idx.display())]),
        });
    }

    // Collect paths to backup.
    let backup_entries: Vec<_> = match args.index {
        Some(ref idx) => store
            .pick_all()
            .paths()
            .filter(|p| p.starts_with(idx))
            .collect(),
        None => store.pick_all().paths().collect(),
    };

    // Create the backup archive.
    let compressed = create_backup_tarball(&backup_entries, args.progress)?;

    // Write to the specified output.
    fs::write(&args.output, compressed, Context::Backup)?;

    Ok(())
}

/// Creates a tarball from which store indices can be restored.
fn create_backup_tarball(
    backup_entries: &Vec<&Path>,
    show_progress: bool,
) -> Result<Vec<u8>, Error> {
    let mut archive = tar::Builder::new(Vec::new());
    let bar = indicatif::ProgressBar::new(1 + backup_entries.len() as u64);

    // Store absolute paths in a friendly format.
    let (manifest, archive_entries) = pack_manifest(backup_entries);
    let manifest_json = serde_json::to_vec(&manifest).map_err(|e| Error::Serialisation {
        cause: e.to_string(),
        context: Context::Backup,
    })?;

    // Embed the manifest into the tarball
    let mut header = tar::Header::new_gnu();
    header.set_size(manifest_json.len() as u64);
    header.set_mode(0o644);
    archive
        .append_data(&mut header, "mk::manifest.json", manifest_json.as_slice())
        .expect("Write on `Vec<u8>` is infallible");

    if show_progress {
        bar.inc(1);
    }

    // Add the safe entries to the tarball
    for entry in archive_entries {
        archive
            .append_path_with_name(unpack_manifest(&manifest, &entry), &entry)
            .inspect_err(|e| eprintln!("{e}"))
            .map_err(|_| Error::FsDenied {
                which: entry.to_path_buf(),
                context: Context::Backup,
            })?;
        if show_progress {
            bar.inc(1);
        }
    }
    let tar_bytes = archive
        .into_inner()
        .expect("Write on `Vec<u8>` is infallible");

    // Compress the tarball
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    _ = encoder.write_all(&tar_bytes);

    Ok(encoder.finish().expect("Write on `Vec<u8>` is infallible"))
}

/// Generates the manifest file and the tarball paths for each recipe, given a list of their paths.
fn pack_manifest(paths: &[&Path]) -> (HashMap<u32, PathBuf>, Vec<PathBuf>) {
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

    (out_manifest, out_paths)
}

/// Gets the "real" absolute path to a tarball entry given its manifest.
///
/// This function can panic for any number of reasons, each of which indicates a corrupt backup:
/// 1. The provided path has no parent directory in the tarball
/// 2. The provided parent directory can't be parsed as a [`u32`]
/// 3. The provided parent directory isn't in the manifest
/// 4. The provided path has no file name component
fn unpack_manifest(manifest: &HashMap<u32, PathBuf>, path: &Path) -> PathBuf {
    let parent = path.parent().unwrap().to_string_lossy().parse().unwrap();
    let real_parent = &manifest[&parent];

    real_parent.join(path.file_name().unwrap())
}
