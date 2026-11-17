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

//! Implementation of `mk store import`.
use super::*;

use crate::cli::style::mkdev_pb_style;
use crate::{error::Context, fs};

use std::io::Read;
use std::path::PathBuf;

use flate2::read::GzDecoder;
use indicatif::ProgressFinish;

#[derive(Parser, Debug)]
pub struct Import {
    /// Backup to restore.
    #[arg(short, long)]
    pub input: PathBuf,

    /// Validate the backup without restoring its contents.
    #[arg(short, long)]
    pub validate: bool,

    /// Display a progress bar (useful for large backups).
    #[arg(short, long)]
    pub progress: bool,

    /// Print verbose information.
    #[arg(short = 'V', long, conflicts_with = "progress")]
    pub verbose: bool,
}

impl MkdevCommand for Import {
    fn execute(self, store: RecipeStore) -> Result<(), Error> {
        import_command(self, store)
    }
}

pub fn import_command(args: Import, mut store: RecipeStore) -> Result<(), Error> {
    let gz_bytes = fs::read(&args.input, Context::Backup)?;
    let mut decoder = GzDecoder::new(gz_bytes.as_slice());
    let mut tar_bytes = Vec::new();

    _ = decoder
        .read_to_end(&mut tar_bytes)
        .map_err(|e| Error::Deserialisation {
            cause: e.to_string(),
            context: Context::Backup,
        })?;

    if args.progress {
        let bar = indicatif::ProgressBar::no_length()
            .with_style(mkdev_pb_style())
            .with_finish(ProgressFinish::Abandon);

        store.install_tar_with_progress(&tar_bytes, |len, target| {
            bar.set_length(len as u64);
            bar.set_message(format!("{}", target.display()));
            bar.inc(1);
        })
    } else if args.verbose {
        store.install_tar_with_progress(&tar_bytes, |_, target| eprintln!("{}", target.display()))
    } else {
        store.install_tar(&tar_bytes)
    }
}
