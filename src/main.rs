mod build_recipe;
mod cli;
mod config;
mod content;
mod delete;
mod imprint;
mod list;
mod recipe;
mod subs;

use build_recipe::build_recipes;
use cli::{Cli, Commands::*};
use delete::delete_recipe;
use imprint::imprint_recipe;
use list::list_recipe;
use recipe::Recipe;

use std::collections::HashMap;

use clap::Parser;

fn main() {
    let args = Cli::parse();

    let recipes = load_user_data();

    let status = try_get_status(args, recipes);

    if let Err(why) = status {
        eprintln!("mkdev: error: {why}");
        std::process::exit(1);
    }
}

/// Load user recipes into a hashmap for easy searching
fn load_user_data() -> HashMap<String, Recipe> {
    let recipes = Recipe::gather().unwrap_or_else(|error| {
        panic!("Error gathering stored recipes: {error:?}");
    });

    recipes
}

/// Dispatcher for various actions
fn try_get_status(args: Cli, recipes: HashMap<String, Recipe>) -> Result<(), String> {
    match args.command {
        Some(command) => match command {
            Imprint {
                recipe,
                description,
            } => imprint_recipe(recipe, description),
            Delete { recipe } => delete_recipe(recipe, &recipes),
            List { recipe } => list_recipe(recipe, &recipes),
        },
        None => build_recipes(args, recipes),
    }
}
