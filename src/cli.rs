#![deny(missing_docs)]
use crate::output_type::OutputType;
use crate::recipe_completer::recipe_completer;

use std::path::PathBuf;

use clap::{crate_authors, crate_description, crate_version, ArgAction, Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;

#[derive(Parser, Debug)]
#[command(
    name = "mk",
    version = crate_version!(),
    long_version = concat!(
        crate_version!(), " — ", crate_description!(),
        "\n© 2025 ", crate_authors!(),
        ".\nLicensed under the MIT License — see https://github.com/4jamesccraven/mkdev/blob/main/LICENSE for details.",
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

    /// Specify configuration file to load.
    #[arg(short, long, env = "CONFIG")]
    pub config: Option<PathBuf>,

    /// Display default config to stdout
    #[arg(short, long)]
    pub gen_config: bool,

    /// Display current config to stdout
    #[arg(short, long)]
    pub print_config: bool,

    /// Displays the manpage
    #[arg(long, hide = true, env = "MANPAGE")]
    pub man_page: bool,
}

#[derive(Subcommand, Debug)]
/// Commands to be passed
pub enum Commands {
    /// Build a recipe/some recipes by name [Aliases: build | invoke]
    #[command(aliases = ["build", "conjure", "invoke", "summon"])]
    Evoke(Evoke),
    /// Create a recipe by cloning the contents of the current directory [Alias: clone]
    #[command(aliases = ["clone"])]
    Imprint(Imprint),
    /// Delete a recipe
    Delete(Delete),
    /// List recipes, or the contents of a specific one [Alias: show]
    #[command(aliases = ["show"])]
    List(List),
}

#[derive(Parser, Clone, Debug)]
pub struct Evoke {
    /// The recipe(s) to build
    #[arg(add = ArgValueCompleter::new(recipe_completer))]
    pub recipes: Vec<String>,

    /// Target directory for recipe output
    #[arg(last = true)]
    pub dir_name: Option<String>,

    /// The 'name' of the instantiated recipe. This replaces substitutions that evaluate to
    /// mk::name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Print debug info during build
    #[arg(short, long)]
    pub verbose: bool,

    /// Supress warnings about destructive actions
    #[arg(short, long)]
    pub suppress_warnings: bool,
}

#[derive(Parser, Debug)]
pub struct Imprint {
    /// The name of the recipe to imprint.
    pub recipe: String,

    /// Description to be associated with recipe
    #[arg(short, long)]
    pub description: Option<String>,

    /// Supress warnings about destructive actions
    #[arg(short, long)]
    pub suppress_warnings: bool,

    /// Write the recipe as a Nix expression & save it to FILE
    #[arg(short = 'n', long, value_name = "FILE")]
    pub to_nix: Option<PathBuf>,

    /// Paths/globs to exclude from the recipe
    #[arg(short, long, value_name = "FILE/GLOB", action = ArgAction::Append)]
    pub exclude: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct Delete {
    /// The recipe to delete
    #[arg(add = ArgValueCompleter::new(recipe_completer))]
    pub recipe: String,
}

#[derive(Parser, Debug)]
pub struct List {
    /// Specific recipe
    #[arg(add = ArgValueCompleter::new(recipe_completer))]
    pub recipe: Option<String>,

    /// Style of output
    #[arg(short, long)]
    pub r#type: Option<OutputType>,
}
