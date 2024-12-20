mod cli;
mod recipe;
mod content;

use cli::{Cli, Commands::*};
use recipe::Recipe;

use std::fmt::{Display, Debug};
use std::collections::HashMap;

use clap::Parser;

fn error_handler<T, E>(res: Result<T, E>) -> T
where
    E: Display + Debug,
{
    if let Err(e) = res {
        eprintln!("{e}");
        std::process::exit(1);
    }
    res.unwrap()
}

fn main() {
    let args = Cli::parse();

    let recipes = Recipe::gather();
    let recipes: HashMap<String, Recipe> = error_handler(recipes)
        .iter()
        .map(|r| {
            (r.name.clone(), r.to_owned())
        })
        .collect();

    if let Some(command) = args.command {
        match command {
            Imprint { recipe, description } => {
                let new = Recipe::imprint(recipe, description);
                let new = error_handler(new);
                let _ = error_handler(new.save());
            },
            Delete { recipe } => {
                let to_delete = recipes.get(recipe.as_str());

                match to_delete {
                    Some(recipe) => error_handler(recipe.delete()),
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
                }
                else {
                    for recipe in recipes.values() {
                        recipe.list(false);
                        println!()
                    }
                }
            }
        }
    }
    else {
        for recipe in args.recipes.unwrap() {
            if !recipes.contains_key(&recipe) {
                eprintln!("No such recipe \"{recipe}\".");
                continue;
            }

            let recipe = recipes.get(&recipe).unwrap();

            let dir = if let Some(dir) = &args.dir_name {
                std::path::PathBuf::from(dir)
            }
            else {
                error_handler(std::env::current_dir())
            };

            error_handler(Recipe::build(&dir, &recipe.contents, args.verbose));
        }
    }
}
