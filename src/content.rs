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

use crate::cli::Imprint;
use crate::fs_wrappers;
use crate::mkdev_error::Context;
use crate::mkdev_error::Error;

use std::cmp::Ordering;
use std::path::Path;
use std::path::PathBuf;

use ignore::{Walk, WalkBuilder};
use serde::{Deserialize, Serialize};

/// The data a mkdev recipe stores.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RecipeItem {
    File(File),
    Directory(PathBuf),
}

impl RecipeItem {
    /// Returns the name of the `RecipeItem`.
    pub fn name(&self) -> String {
        let name = match self {
            RecipeItem::File(file) => file.name.to_string_lossy(),
            RecipeItem::Directory(dir) => dir.to_string_lossy(),
        };

        name.into()
    }

    /// Constructs a new `RecipeItem::Directory` variant
    fn dir(name: PathBuf) -> Self {
        Self::Directory(name)
    }
}

/// A file.
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct File {
    pub name: PathBuf,
    pub content: String,
}

/// Recursively detects and saves every file and subdirectory in the current working directory.
///
/// Standard ignore filters are applied (.gitignore, .ignore, etc.), and symlinks are ignored.
pub fn make_contents(walk: Walk, root: &Path) -> Result<Vec<RecipeItem>, Error> {
    let mut out = vec![];

    for file in walk.flatten() {
        if file.path() == root {
            continue;
        }

        let data = file
            .file_type()
            .expect("only none for stdin, which is not allowed.");

        let mut path = file.into_path();

        if path.starts_with(root) {
            path = path
                .strip_prefix(root)
                .expect("prefix is confirmed to exist")
                .into();
        }

        match data {
            data if data.is_symlink() => continue, // ignore symlinks
            data if data.is_dir() => out.push(RecipeItem::dir(path)),
            data if data.is_file() => out.push(RecipeItem::File(File {
                name: path.clone(),
                content: fs_wrappers::read_to_string(root.join(path), Context::Imprint)?,
            })),

            // All of these methods' results are mutually exclusive and exhaustive.
            // see: https://doc.rust-lang.org/nightly/std/fs/struct.FileType.html
            _ => unreachable!(),
        }
    }

    out.sort();
    Ok(out)
}

/// Constructs a recursive walk.
///
/// This function exists to allow users to override the walk behaviour.
pub fn build_walk(args: &Imprint) -> Result<Walk, Error> {
    let cwd = fs_wrappers::current_dir()?;

    let mut ob = OverrideBuilder::new(&cwd);
    for over in &args.exclude {
        ob.add(&format!("!{over}"))?;
    }
    let user_filters = ob.build()?;

    Ok(WalkBuilder::new(&cwd)
        .standard_filters(!args.no_filter)
        .overrides(user_filters)
        .build())
}

impl std::fmt::Display for RecipeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl PartialEq for RecipeItem {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for RecipeItem {}

impl PartialOrd for RecipeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RecipeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        fn key(r: &RecipeItem) -> (bool, String) {
            (matches!(r, RecipeItem::File(_)), r.name())
        }

        key(self).cmp(&key(other))
    }
}

impl std::hash::Hash for RecipeItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RecipeItem::File(f) => {
                0u8.hash(state);
                f.name.hash(state);
            }
            RecipeItem::Directory(d) => {
                1u8.hash(state);
                d.hash(state);
            }
        }
    }
}
