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

//! Functions for instantiating recipes in the local environment.
use super::RecipeItem;

use crate::error::{Context, Error};
use crate::fs;

use std::path::PathBuf;

/// Builds a single recipe by taking in its contents and writing them to disk.
///
/// If a file in the target directory already exists and the conflict resolution strategy is not
/// `Overwrite`, a Destruction error is returned.
pub fn instantiate_contents(contents: &[RecipeItem], opts: &InstantiateOpts) -> Result<(), Error> {
    contents.iter().try_for_each(|content| {
        let dest = opts.root_dir.join(content.as_path());
        if let Some(parent) = &dest.parent() {
            fs::create_dir_all(parent, Context::Evoke)?;
        }

        if opts.verbose {
            eprintln!("{}", dest.display());
        }

        match content {
            RecipeItem::File(file) => {
                // Stop if a file would be overwritten unless the user has explicitly suppressed
                // it.
                match opts.on_conflict {
                    OnConflict::Guard if dest.is_file() => Err(Error::DestructiveAction {
                        name: dest.to_string_lossy().into(),
                    }),
                    _ => fs::write(&dest, &file.content, Context::Evoke),
                }
            }
            RecipeItem::Directory(_) => fs::create_dir_all(&dest, Context::Evoke),
        }
    })
}

/// Defines how existing files should be handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnConflict {
    /// Replace existing files.
    Overwrite,
    /// Preserve existing files, refusing to overwrite.
    Guard,
}

impl OnConflict {
    /// Returns [`OnConflict::Overwrite`] if `yes` is `true`, or [`OnConflict::Guard`] otherwise.
    #[inline]
    pub fn allow_overwrite(yes: bool) -> Self {
        if yes { Self::Overwrite } else { Self::Guard }
    }
}

/// Options for instantiating [`RecipeItem`]s.
#[derive(Clone, Debug)]
pub struct InstantiateOpts {
    /// The root directory where contents are placed.
    /// Default: "."
    pub root_dir: PathBuf,
    /// The conflict resolution strategy.
    /// Default: `Guard`
    pub on_conflict: OnConflict,
    /// Whether to print verbose construction info.
    /// Default: false
    pub verbose: bool,
}

impl Default for InstantiateOpts {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            on_conflict: OnConflict::Guard,
            verbose: false,
        }
    }
}
