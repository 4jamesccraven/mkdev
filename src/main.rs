mod cli;
mod config;
mod content;
mod recipe;
mod subs;

use cli::{Cli, Commands::*};
use recipe::Recipe;
use subs::Replacer;

use std::collections::HashMap;

use clap::Parser;

fn main() {
    let args = Cli::parse();

    let recipes = Recipe::gather().unwrap_or_else(|error| {
        panic!("Error gathering stored recipes: {error:?}");
    });
    let recipes: HashMap<String, Recipe> = recipes
        .iter()
        .map(|r| (r.name.clone(), r.to_owned()))
        .collect();

    if let Some(command) = args.command {
        match command {
            Imprint {
                recipe,
                description,
            } => {
                let new = Recipe::imprint(recipe, description).unwrap_or_else(|error| {
                    panic!("Unable to read current_working directory for the recipe: {error:?}");
                });
                new.save().unwrap_or_else(|error| {
                    panic!("Unable to save instantiated recipe: {error:?}");
                });
            }
            Delete { recipe } => {
                let to_delete = recipes.get(recipe.as_str());

                match to_delete {
                    Some(recipe) => recipe.delete().unwrap_or_else(|error| {
                        panic!("Unable to delete `{}`: {error:?}", recipe.name);
                    }),
                    None => eprintln!("No such recipe \"{recipe}\"."),
                }
            }
            List { recipe } => {
                if let Some(recipe) = recipe {
                    let to_show = recipes.get(recipe.as_str());

                    match to_show {
                        Some(recipe) => recipe.list(true),
                        None => eprintln!("No such recipe \"{recipe}\"."),
                    }
                } else {
                    for recipe in recipes.values() {
                        recipe.list(false);
                        println!()
                    }
                }
            }
        }
    } else {
        let rec_args = args.recipes.unwrap();

        let mut can_proceed = true;
        for recipe in rec_args.clone() {
            if !recipes.contains_key(&recipe) {
                eprintln!("No such recipe \"{recipe}\".");
                can_proceed = false;
            }
        }

        if !can_proceed {
            std::process::exit(1);
        }

        for recipe in rec_args.clone() {
            let recipe = recipes.get(&recipe).unwrap();

            let dir = if let Some(dir) = &args.dir_name {
                std::path::PathBuf::from(dir)
            } else {
                std::env::current_dir().unwrap_or_else(|error| {
                    panic!("Unable to get cwd: {error:?}");
                })
            };

            let re = Replacer::new();

            Recipe::build(&dir, &recipe.contents, args.verbose, &re).unwrap_or_else(|error| {
                panic!(
                    "Unable to write `{}` to {}: {error:?}",
                    recipe.name,
                    dir.display()
                );
            });
        }
    }
}
