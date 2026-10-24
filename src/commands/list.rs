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
//! Implementation of `mk list`.
//!
//! Supports printing single recipes or all known recipes in various formats.
use crate::cli::List;
use crate::config::Config;
use crate::error::{Error, Subject};
use crate::output_type::OutputType::{self, *};
use crate::presentation::{format_recipe_list, recipe_treefmt};
use crate::recipe::Recipe;
use crate::recipe::RecipeStore;

use rust_i18n::t;

/// List a recipe/recipes in accordance to the provide command line arguments.
pub fn list_recipe(args: List, store: RecipeStore) -> Result<(), Error> {
    let output_type = args.r#type.unwrap_or_default();

    let query = match args.recipe {
        Some(ref id) => store.pick_one(id)?,
        None => store.pick_all(),
    };

    let mut selection: Vec<_> = store.select(&query).map(|m| &m.recipe).collect();
    selection.sort_by_key(|&r| (r.namespace(), r.shortname()));

    if let Some(id) = args.recipe {
        match selection.len() {
            1 => display_one(selection[0], output_type),
            0 => {
                return Err(Error::Invalid {
                    subject: Subject::Recipe,
                    examples: Some(vec![id]),
                });
            }
            _ => unreachable!(),
        }
    } else {
        display_all(selection, output_type, !args.no_description);
    }

    Ok(())
}

const SER_EXISTING_RECIPE: &str = //.
    "Invalid recipes are filtered out by this point, \
     and if they deserialised, they'll serialise back.";

/// Displays all recipes.
fn display_all(recipes: Vec<&Recipe>, output_type: OutputType, show_description: bool) {
    match output_type {
        Default => {
            let mut display_cfg = Config::get()
                .expect("config is guaranteed to be set")
                .recipe_fmt
                .clone();

            if display_cfg.show_descriptions.is_none() {
                display_cfg.show_descriptions = Some(show_description)
            }

            print!("{}", format_recipe_list(&recipes, &display_cfg));
        }
        Debug => recipes.iter().for_each(|r| println!("{:#?}", r)),
        Plain => recipes.iter().for_each(|r| println!("{}", r.name)),
        Print0 => print!(
            "{}",
            recipes
                .iter()
                .map(|r| r.name.to_string())
                .collect::<Vec<_>>()
                .join("\0")
        ),
        Json => println!(
            "{}",
            serde_json::to_string_pretty(&recipes).expect(SER_EXISTING_RECIPE)
        ),
        Toml => {
            recipes.iter().for_each(|r| {
                println!("{}", toml::to_string_pretty(r).expect(SER_EXISTING_RECIPE))
            });
            crate::warning!("{}", t!("warnings.toml_all"));
        }
        Nix => println!(
            "{}",
            ser_nix::to_string(&recipes).expect(SER_EXISTING_RECIPE)
        ),
    }
}

fn display_one(recipe: &Recipe, output_type: OutputType) {
    match output_type {
        Default => print!("{}", recipe_treefmt(recipe)),
        Debug => println!("{:#?}", recipe),
        Plain => println!("{}", content_names_joined(recipe, "\n")),
        Print0 => print!("{}", content_names_joined(recipe, "\0")),
        Json => println!(
            "{}",
            serde_json::to_string_pretty(recipe).expect(SER_EXISTING_RECIPE)
        ),
        Toml => println!(
            "{}",
            toml::to_string_pretty(recipe).expect(SER_EXISTING_RECIPE)
        ),
        Nix => println!(
            "{}",
            ser_nix::to_string(&recipe).expect(SER_EXISTING_RECIPE)
        ),
    }
}

fn content_names_joined(recipe: &Recipe, sep: &str) -> String {
    let mut names = recipe.contents.iter().map(|c| c.name()).collect::<Vec<_>>();
    names.sort();

    names.join(sep)
}
