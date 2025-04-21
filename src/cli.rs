use crate::recipe_completer::recipe_completer;

use clap::{crate_authors, crate_description, crate_version, Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;

#[derive(Parser, Debug)]
#[command(
    name = "mk",
    version = crate_version!(),
    long_version = concat!(
        crate_version!(), " — ", crate_description!(), "\n",
        "© 2025 ", crate_authors!(), ". Licensed under MIT License — see LICENSE for details."
    ),
    author = crate_authors!(),
    about = crate_description!(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build a recipe/some recipes by name
    #[command(aliases = ["build", "b", "conjure", "summon", "invoke"])]
    Evoke {
        /// The recipe(s) to build
        #[arg(add = ArgValueCompleter::new(recipe_completer))]
        recipes: Vec<String>,

        /// Target directory for recipe output
        #[arg(last = true)]
        dir_name: Option<String>,

        /// Prints debug info during build
        #[arg(short, long)]
        verbose: bool,
    },
    /// Create a recipe by "imprinting" the contents
    /// of the current directory
    #[command(aliases = ["clone", "i"])]
    Imprint {
        /// The name of the recipe to imprint.
        /// NOTE: this action IS destructive and
        /// can overwrite existing recipes
        recipe: String,

        #[arg(short, long)]
        /// Description to be associated with recipe
        description: Option<String>,
    },
    /// Delete a recipe
    Delete {
        /// The recipe to delete
        #[arg(add = ArgValueCompleter::new(recipe_completer))]
        recipe: String,
    },
    /// List recipes, or the contents of a specific one
    List {
        #[arg(add = ArgValueCompleter::new(recipe_completer))]
        recipe: Option<String>,
    },
}
