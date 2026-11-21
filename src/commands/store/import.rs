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
use crate::gz;
use crate::{error::Context, fs};

use std::path::PathBuf;

use indicatif::ProgressFinish;

#[derive(Parser, Debug)]
pub struct Import {
    /// Backup to restore.
    #[arg(short, long)]
    pub input: PathBuf,

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
    let tar_bytes = gz::decode_bytes(&gz_bytes)?;

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
