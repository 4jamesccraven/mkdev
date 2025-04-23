use crate::output_type::OutputType;
use crate::recipe_completer::recipe_completer;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;

include!(concat!(env!("OUT_DIR"), "/built_metadata.rs"));

#[deny(missing_docs)]
#[derive(Parser, Debug)]
#[command(
    name = "mk",
    version = VERSION,
    long_version = LONG_VERSION,
    author = AUTHORS,
    about = DESCRIPTION,
)]
/// Command Line Interface for Mkdev
pub struct Cli {
    /// Command to be passed
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Specifies configuration file to load
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Generate default standard config to stdout
    #[arg(short, long)]
    pub gen_config: bool,

    /// Displays current config to stdout
    #[arg(short, long)]
    pub print_config: bool,
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
        /// Specific recipe
        #[arg(add = ArgValueCompleter::new(recipe_completer))]
        recipe: Option<String>,
        /// Style of output
        #[arg(short, long)]
        r#type: Option<OutputType>,
    },
}
