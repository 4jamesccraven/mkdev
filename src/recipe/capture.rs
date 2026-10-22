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
//! Utilties for constructing and manipulating the contents of a mkdev recipe.
use ignore::overrides::OverrideBuilder;

use crate::error::Context;
use crate::error::Error;
use crate::fs;
use crate::recipe::RecipeItem;

use std::path::{Path, PathBuf};

use ignore::{Walk, WalkBuilder};

/// Create a [`Vec`] of [`RecipeItem`] based on the contents of the provided path.
///
/// Some files are excluded, see [`CaptureOpts`]. For details, see [`ignore::Walk`].
pub fn capture_contents<P>(root: P) -> Result<Vec<RecipeItem>, Error>
where
    P: AsRef<Path>,
{
    let opts = CaptureOpts {
        root: PathBuf::from(root.as_ref()),
        ..Default::default()
    };

    capture_contents_with(opts)
}

/// Walk a directory collecting files and directories as [`RecipeItem`]s.
///
/// The directory, as well as the types of filters to apply, are specified via the `opts` parameter.
/// For details, see [`CaptureOpts`].
pub fn capture_contents_with(opts: CaptureOpts) -> Result<Vec<RecipeItem>, Error> {
    let root = opts.root.clone();
    let walk = Walk::try_from(opts)?;

    walk.flatten()
        .filter(|e| (e.path() != root) && !e.path_is_symlink())
        .filter_map(|entry| -> Option<Result<RecipeItem, Error>> {
            // This will never be `None` unless `entry` is stdin.
            let data = entry.file_type()?;

            // Get the path relative to root.
            let raw_path = entry.into_path();
            let path = raw_path.strip_prefix(&root).unwrap_or(&raw_path).to_owned();

            match data {
                data if data.is_dir() => Some(Ok(RecipeItem::dir(path))),
                // This line will need to break if we can't read the contents of path to a string
                data if data.is_file() => {
                    let try_init = fs::read_to_string(root.join(&path), Context::Imprint)
                        .map(|content| RecipeItem::file(path, content));
                    Some(try_init)
                }
                // The only possibilities are is_dir, is_file, or is_symlink; and is_symlink is
                // filtered out by this point.
                // see: https://doc.rust-lang.org/nightly/std/fs/struct.FileType.html
                _ => unreachable!(),
            }
        })
        .collect::<Result<Vec<_>, Error>>()
        .map(|mut rs| {
            rs.sort();
            rs
        })
}

/// Options for capturing the contents of a directory.
#[derive(Clone, Debug)]
pub struct CaptureOpts {
    /// The directory to walk.
    /// Default: "."
    pub root: PathBuf,
    /// Whether to apply standard filters.
    /// Default: true
    pub filtered: bool,
    /// Extra globs to exclude from the walk.
    /// Default: []
    pub overrides: Vec<String>,
}

impl Default for CaptureOpts {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            filtered: true,
            overrides: vec![],
        }
    }
}

impl TryFrom<CaptureOpts> for Walk {
    type Error = Error;

    fn try_from(opts: CaptureOpts) -> Result<Self, Self::Error> {
        let mut wb = WalkBuilder::new(opts.root.as_path());
        wb.standard_filters(opts.filtered);

        if !opts.overrides.is_empty() {
            let mut ob = OverrideBuilder::new(opts.root.as_path());
            opts.overrides
                .into_iter()
                .try_for_each(|over| ob.add(&over).map(|_| ()))?;
            wb.overrides(ob.build()?);
        }

        Ok(wb.build())
    }
}
