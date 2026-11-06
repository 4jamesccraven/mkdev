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

//! Implementation of `mk store scry`.
//!
//! This command has output similar to GNU stat.
use super::*;

use std::fmt::Display;

use rust_i18n::t;

#[derive(Parser, Clone, Debug)]
pub struct Scry {
    /// The recipe to display.
    pub recipe: String,
}

impl MkdevCommand for Scry {
    fn execute(self, store: RecipeStore) -> Result<(), Error> {
        scry_command(self, store)
    }
}

pub fn scry_command(args: Scry, store: RecipeStore) -> Result<(), Error> {
    let id = store.pick_id(args.recipe)?;
    // TODO: put getter on store that avoids this clone
    let entry = store.pick_one(id.clone()).unwrap();

    // Size in bytes of recipe file
    let size = format!("{:<10}", DisplayQuestion(store.file_len(id)));
    // Number of file and directory `RecipeItem`s in recipe
    let (files, dirs) = store.content_breakdown(id).unwrap();
    // List of languages in metadata
    let langs = store
        .recipe_langs(id)
        .unwrap()
        .iter()
        .map(|l| l.name.to_owned())
        .collect::<Vec<_>>()
        .join(", ");
    // Whether the recipe is externally managed.
    let is_external = DisplayQuestion(store.is_external(id).map(|yes| {
        if yes {
            t!("general.yes")
        } else {
            t!("general.no")
        }
    }))
    .to_string();

    // Group in rows.
    let rows = [
        vec![StatField {
            label: t!("general.path").to_string(),
            value: format!("{}", entry.store_path.as_path().display()),
        }],
        vec![
            StatField {
                label: t!("general.size").to_string(),
                value: size.to_string(),
            },
            StatField {
                label: t!("general.external").to_string(),
                value: is_external,
            },
        ],
        vec![StatField {
            label: "ID".into(),
            value: id.to_string(),
        }],
        vec![
            StatField {
                label: t!("general.files").to_string(),
                value: format!("{files:<10}"),
            },
            StatField {
                label: t!("general.directories").to_string(),
                value: format!("{dirs:<10}"),
            },
        ],
        vec![StatField {
            label: "Langs".to_string(),
            value: langs,
        }],
    ];

    // Max length of first label in each row
    let label_width = rows
        .iter()
        .filter_map(|fields| fields.first())
        .map(|field| field.label.len())
        .max()
        .unwrap_or_default();

    for row in rows {
        // Align key of first field in row only.
        if let Some((first, rest)) = row.split_first() {
            print!("{:>label_width$}: {} ", first.label, first.value);

            for field in rest {
                print!("{}: {} ", field.label, field.value);
            }
        }
        println!();
    }

    Ok(())
}

struct StatField {
    label: String,
    value: String,
}

struct DisplayQuestion<T>(Option<T>);

impl<T: Display> Display for DisplayQuestion<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(ref val) => f.pad(&val.to_string()),
            None => f.pad("??"),
        }
    }
}
