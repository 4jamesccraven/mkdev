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
use super::Recipe;
use crate::cli::List;
use crate::config::Config;
use crate::display::{display_recipes_with_config, repr_tree};
use crate::mkdev_error::Error;
use crate::output_type::OutputType::{self, *};

use std::collections::HashMap;

use colored::Colorize;
use rust_i18n::t;

/// List a recipe/recipes in accordance to the provide command line arguments.
pub fn list_recipe(args: List, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let output_type = args.r#type.unwrap_or_default();

    match args.recipe {
        Some(recipe) => {
            display_one(Recipe::pick(&user_recipes, &recipe)?, output_type);
        }
        None => {
            let mut recipes: Vec<_> = user_recipes.values().collect();
            recipes.sort_by_key(|&r| (r.namespace().is_some(), r.shortname()));

            display_all(recipes, output_type, !args.no_description);
        }
    }

    Ok(())
}

const SER_EXISTING_RECIPE: &str = //.
    "Invalid recipes are filtered out by this point, \
     and if they deserialised, they'll serialise back.";

/// Displays all recipes.
fn display_all(recipes: Vec<&Recipe>, output_type: OutputType, show_description: bool) {
    let mut config = Config::get()
        .expect("config is guaranteed to be set")
        .recipe_fmt
        .clone();

    if config.show_descriptions.is_none() {
        config.show_descriptions = Some(show_description)
    }

    match output_type {
        Default => print!("{}", display_recipes_with_config(&recipes, &config)),
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
            println!(
                "{}",
                recipes
                    .iter()
                    .map(|r| toml::to_string_pretty(r).expect(SER_EXISTING_RECIPE))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
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
        Default => print!("{}", recipe.display_contents()),
        Debug => println!("{:#?}", recipe),
        Plain => print!("{}", recipe.display_contents_plain("\n")),
        Print0 => print!("{}", recipe.display_contents_plain("\0")),
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

impl Recipe {
    /// Display the recipe's contents in a tree format.
    pub fn display_contents(&self) -> String {
        let mut out = format!("{}\n", self.name.bold().blue());
        let contents = repr_tree(&self.contents);
        out.push_str(&contents);

        out
    }

    /// Display the name of all the recipe's contents.
    pub fn display_contents_plain(&self, sep: &str) -> String {
        let mut names = self.contents.iter().map(|c| c.name()).collect::<Vec<_>>();
        names.sort();

        names.join(sep)
    }
}
