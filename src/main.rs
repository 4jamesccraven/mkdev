mod config_hook;
mod delete;
mod evoke;
mod imprint;
mod list;

use config_hook::config_hook;
use delete::delete_recipe;
use evoke::build_recipes;
use imprint::imprint_recipe;
use list::list_recipe;

use mkdev_cli::cli::{Cli, Commands::*};
use mkdev_cli::man::man_env;
use mkdev_recipe::recipe::Recipe;

use std::collections::HashMap;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;

fn main() {
    CompleteEnv::with_factory(Cli::command).complete();
    man_env();

    let args = Cli::parse();

    let status = try_get_status(args);

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
fn try_get_status(args: Cli) -> Result<(), String> {
    config_hook(&args);

    let user_recipes = load_user_data();

    match args.command {
        Some(command) => match command {
            Evoke {
                recipes,
                dir_name,
                verbose,
            } => build_recipes(recipes, dir_name, verbose, user_recipes),
            Imprint {
                recipe,
                description,
            } => imprint_recipe(recipe, description),
            Delete { recipe } => delete_recipe(recipe, &user_recipes),
            List { recipe, r#type } => list_recipe(recipe, r#type, &user_recipes),
        },
        None => Err("No action specified.".into()),
    }
}
