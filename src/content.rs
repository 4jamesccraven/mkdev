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
use crate::error::Context;
use crate::error::Error;
use crate::fs;
use crate::recipe::RecipeItem;

use std::path::Path;

use ignore::{Walk, WalkBuilder};

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
            data if data.is_file() => out.push(RecipeItem::file(
                path.clone(),
                fs::read_to_string(root.join(path), Context::Imprint)?,
            )),

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
    let cwd = fs::current_dir()?;

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
