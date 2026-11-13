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
use super::*;

use crate::cli::completion::index_completer;
use crate::error::{Context, Subject};
use crate::fs;

use std::io::Write;
use std::path::PathBuf;

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
    let backup_indices: Vec<_> = match args.index {
        Some(idx) => vec![idx],
        None => store.pick_indices().cloned().collect(),
    };

    // Build the tarball for the backup
    let bar = indicatif::ProgressBar::new(backup_indices.len() as u64);
    let tar_bytes = if args.progress {
        store.tar_indices_with_progress(&backup_indices, |_| bar.inc(1))
    } else {
        store.tar_indices(&backup_indices)
    }?;

    // Compress the tarball
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&tar_bytes)
        .map_err(|e| Error::Serialisation {
            cause: e.to_string(),
            context: Context::Backup,
        })?;
    let compressed = encoder.finish().expect("Write on `Vec<u8>` is infallible");

    // Write to the specified output.
    fs::write(&args.output, compressed, Context::Backup)?;

    Ok(())
}
