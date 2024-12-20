mod cli;
mod config;

use cli::{Cli, Commands::*};
use config::Recipe;

use std::fmt::{Display, Debug};

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
    let recipes = error_handler(recipes);

    if let Some(command) = args.command {
        match command {
            Imprint { recipe, description } => {
                eprintln!("{recipe}, {description:?}");
                let new = Recipe::imprint(recipe, description);
                let new = error_handler(new);
                let _ = error_handler(new.save());
            },
            Delete { recipe } => {
                let to_delete = recipes
                    .iter()
                    .find(|r| {
                        r.name == recipe
                    });

                match to_delete {
                    Some(recipe) => error_handler(recipe.delete()),
                    None => eprintln!("No such recipe \"{recipe}\"."),
                }
            }
        }
    }
}
