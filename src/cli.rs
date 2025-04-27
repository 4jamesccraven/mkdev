use crate::output_type::OutputType;
use crate::recipe_completer::recipe_completer;

use std::path::PathBuf;

use clap::{crate_authors, crate_description, crate_version, Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;

#[deny(missing_docs)]
#[derive(Parser, Debug)]
#[command(
    name = "mk",
    version = crate_version!(),
    long_version = concat!(
        crate_version!(), " — ", crate_description!(),
        "\n© 2025 ", crate_authors!(),
        ".\nLicensed under MIT License — see https://github.com/4jamesccraven/mkdev/blob/main/LICENSE for details.",
    ),
    author = crate_authors!(),
    about = crate_description!(),
    disable_help_subcommand = true,
)]
/// Command Line Interface for Mkdev
pub struct Cli {
    /// Command to be passed
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Specifies configuration file to load.
    #[arg(short, long, env = "CONFIG")]
    pub config: Option<PathBuf>,

    /// Generate default standard config to stdout
    #[arg(short, long)]
    pub gen_config: bool,

    /// Displays current config to stdout
    #[arg(short, long)]
    pub print_config: bool,

    /// Displays the manpage
    #[arg(long, hide = true, env = "MANPAGE")]
    pub man_page: bool,
}

#[deny(missing_docs)]
#[derive(Subcommand, Debug)]
/// Commands to be passed
pub enum Commands {
    /// Build a recipe/some recipes by name. Aliases: [build | invoke]
    #[command(aliases = ["build", "conjure", "invoke", "summon"])]
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
    /// Create a recipe by cloning the contents of the current directory. Aliases: [clone]
    #[command(aliases = ["clone"])]
    Imprint {
        /// The name of the recipe to imprint.
        /// NOTE: this action IS destructive and can overwrite existing recipes
        #[arg(verbatim_doc_comment)]
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
    /// List recipes, or the contents of a specific one. Aliases: [show]
    #[command(aliases = ["show"])]
    List {
        /// Specific recipe
        #[arg(add = ArgValueCompleter::new(recipe_completer))]
        recipe: Option<String>,
        /// Style of output
        #[arg(short, long)]
        r#type: Option<OutputType>,
    },
}
