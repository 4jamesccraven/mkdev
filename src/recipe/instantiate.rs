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
//! ...
use super::RecipeItem;

use crate::error::{Context, Error};
use crate::fs;

use std::path::Path;

/// What should happen if a file already exists in the target directory during evocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnConflict {
    Overwrite,
    Guard,
}

/// Builds a single recipe by taking in its contents and writing them to disk.
///
/// If a file in the target directory already exists and the conflict resolution strategy is not
/// `Overwrite`, a Destruction error is returned.
pub fn instantiate_contents(
    dir: &Path,
    contents: &[RecipeItem],
    on_conflict: OnConflict,
    verbose: bool,
) -> Result<(), Error> {
    contents.iter().try_for_each(|content| {
        let dest = dir.join(content.name());
        if let Some(parent) = &dest.parent() {
            fs::create_dir_all(parent, Context::Evoke)?;
        }

        if verbose {
            eprintln!("{}", dest.display());
        }

        match content {
            RecipeItem::File(file) => {
                // Stop if a file would be overwritten unless the user has explicitly suppressed
                // it.
                match on_conflict {
                    OnConflict::Guard if dest.is_file() => Err(Error::DestructionWarning {
                        name: dest.to_string_lossy().into(),
                    }),
                    _ => fs::write(&dest, &file.content, Context::Evoke),
                }
            }
            RecipeItem::Directory(_) => fs::create_dir_all(&dest, Context::Evoke),
        }
    })
}
