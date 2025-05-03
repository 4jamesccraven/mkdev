mod cli;
mod config;
mod config_hook;
mod content;
mod delete;
mod evoke;
mod imprint;
mod list;
mod man;
mod output_type;
mod recipe;
mod recipe_completer;
mod subs;

use delete::delete_recipe;
use evoke::build_recipes;
use imprint::imprint_recipe;
use list::list_recipe;

use cli::{Cli, Commands::*};
use recipe::Recipe;

use std::collections::HashMap;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;

fn main() {
    // Produce completion scripts using clap_complete...
    CompleteEnv::with_factory(Cli::command).complete();
    // ... or try to do mkdev's business logic
    let status = try_get_status(Cli::parse());

    // Gracefully inform user of error, then exit with fail code
    if let Err(why) = status {
        eprintln!("mkdev: error: {why}");
        std::process::exit(1);
    }
}

/// Load user recipes into a hashmap for easy searching
fn load_user_data() -> HashMap<String, Recipe> {
    let recipes = Recipe::gather().unwrap_or_else(|error| {
        eprintln!("Error gathering stored recipes: {error}");
        std::process::exit(1);
    });

    recipes
}

/// Dispatcher for various actions
fn try_get_status(args: Cli) -> Result<(), String> {
    // Arguments that cause an exit before subcommand logic
    man::hook(&args);
    config_hook::hook(&args);

    let user_recipes = load_user_data();

    match args.command {
        Some(command) => match command {
            Evoke {
                recipes,
                dir_name,
                verbose,
                supress_warnings,
            } => build_recipes(recipes, dir_name, verbose, supress_warnings, user_recipes),
            Imprint {
                recipe,
                description,
                supress_warnings,
            } => imprint_recipe(recipe, description, supress_warnings, user_recipes),
            Delete { recipe } => delete_recipe(recipe, user_recipes),
            List { recipe, r#type } => list_recipe(recipe, r#type, user_recipes),
        },
        None => Err("No action specified.".into()),
    }
}
