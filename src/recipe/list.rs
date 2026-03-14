//! Implementation of `mk list`.
//!
//! Supports printing single recipes or all known recipes in various formats.
use super::Recipe;
use crate::cli::List;
use crate::config::Config;
use crate::display::{display_recipes_with_config, repr_tree};
use crate::mkdev_error::Error::{self, *};
use crate::output_type::OutputType::{self, *};
use crate::warning;

use std::collections::HashMap;

use colored::Colorize;

/// List a recipe/recipes in accordance to the provide command line arguments.
pub fn list_recipe(args: List, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let output_type = args.r#type.unwrap_or_default();

    match args.recipe {
        Some(recipe) => {
            let recipe = user_recipes
                .get(recipe.as_str())
                .ok_or_else(|| Invalid("recipe".into(), Some(vec![recipe])))?;

            display_one(recipe, output_type);
        }
        None => {
            let mut recipes: Vec<_> = user_recipes.values().collect();
            recipes.sort_by(|a, b| a.name.cmp(&b.name));

            display_all(recipes, output_type, !args.no_description);
        }
    }

    Ok(())
}

/// Displays all recipes.
fn display_all(recipes: Vec<&Recipe>, output_type: OutputType, show_description: bool) {
    if let Toml = output_type {
        warning!("option \"TOML\" invalid for displaying multiple recipes. ");
        return;
    }

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
        Json => println!(
            "{}",
            serde_json::to_string_pretty(&recipes)
                .expect("Recipes are instantiated with serde, and should unwrap")
        ),
        Nix => println!(
            "{}",
            ser_nix::to_string(&recipes)
                .expect("Recipes are instantiated with serde, and should unwrap")
        ),
        _ => unreachable!(),
    }
}

fn display_one(recipe: &Recipe, output_type: OutputType) {
    match output_type {
        Default => print!("{}", recipe.display_contents()),
        Debug => println!("{:#?}", recipe),
        Plain => print!("{}", recipe.display_contents_plain()),
        Json => println!(
            "{}",
            serde_json::to_string_pretty(recipe)
                .expect("Recipes are instantiated with serde, and should unwrap")
        ),
        Toml => println!(
            "{}",
            toml::to_string_pretty(recipe)
                .expect("Recipes are instantiated with serde, and should unwrap")
        ),
        Nix => println!(
            "{}",
            ser_nix::to_string(&recipe)
                .expect("Recipes are instantiated with serde, and should unwrap")
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
    pub fn display_contents_plain(&self) -> String {
        let mut names = self.contents.iter().map(|c| c.name()).collect::<Vec<_>>();
        names.sort();

        names.join("\n")
    }
}
