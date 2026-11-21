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

//! Implementation of `mk store check`.
use super::*;

use crate::error::Error;

#[derive(Parser, Debug)]
pub struct Check {
    /// Allow older recipes to retain older formats.
    ///
    /// Without this option, older formats get converted to newer ones.
    #[arg(short = 'D', long, conflicts_with = "show_kind")]
    pub ignore_deprecated: bool,

    /// Allow recipes files to differ from their IDs.
    ///
    /// Without this option, a recipe `foo` in `bar.toml` gets moved to `foo.toml`.
    #[arg(short = 'F', long, conflicts_with = "show_kind")]
    pub ignore_filename: bool,

    /// Print the backing format of each recipe and exit.
    ///
    /// This skips other checks, and writes no changes.
    #[arg(short = 'k', long)]
    pub show_kind: bool,

    /// Write fixes to the store.
    ///
    /// It is recommended to first create a backup with `mk store export`.
    #[arg(short = 'W', long, conflicts_with = "show_kind")]
    pub write_changes: bool,
}

impl MkdevCommand for Check {
    fn execute(self, store: RecipeStore) -> Result<(), Error> {
        check_command(self, store)
    }
}

pub fn check_command(args: Check, mut store: RecipeStore) -> Result<(), Error> {
    let manifest_copy: Vec<_> = store
        .pick_values()
        .map(|(i, e)| (i.clone(), e.clone()))
        .collect();

    let label_width = store
        .pick_all()
        .id_strings()
        .map(String::len)
        .max()
        .unwrap_or_default();

    if args.show_kind {
        manifest_copy
            .into_iter()
            .for_each(|(id, e)| println!("{id:>label_width$}: {}", e.kind));
        return Ok(());
    }

    for (id, e) in manifest_copy {
        let recipe = &e.recipe;
        let store_path = &e.store_path;
        let kind = &e.kind;

        let filename_change = (!args.ignore_filename && !store.filename_matches_id(&id)?)
            .then(|| RecipeStore::store_name(recipe));

        let modernise_outdated = !args.ignore_deprecated && !store.recipe_is_modern(&id)?;

        match filename_change {
            Some(ref new) if !args.write_changes => eprintln!(
                "{id:>label_width$}: filename is '{}', should be '{}'",
                store_path.display(),
                new.display(),
            ),
            Some(ref new) => store.rename_file(&id, new)?,
            None => {}
        }

        if modernise_outdated {
            if !args.write_changes {
                eprintln!("{id:>label_width$}: outdated version: {kind}");
            } else {
                store.replace(&id, e.recipe)?;
            }
        }
    }

    Ok(())
}
