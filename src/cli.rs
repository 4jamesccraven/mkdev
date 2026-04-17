// mkdev - Save your boilerplate instead of writing it
// Copyright (C) 2026  James C. Craven <4jamesccraven@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//! The command line interface for mkdev.
#![deny(missing_docs)]
use crate::output_type::OutputType;
use crate::recipe_completer::recipe_completer;
use crate::{mkdev_error::Error, recipe::Recipe};

use std::{collections::HashMap, path::PathBuf};

use clap::{
    ArgAction, CommandFactory, Parser, Subcommand, crate_authors, crate_description, crate_version,
};
use clap_complete::engine::ArgValueCompleter;

#[derive(Parser, Debug)]
#[command(
    name = "mk",
    version = crate_version!(),
    long_version = concat!(
        crate_version!(), " — ", crate_description!(),
        "\n© 2026 ", crate_authors!(),
        ".\nThis program is free software and comes with ABSOLUTELY NO WARRANTY.",
        "\nYou are welcome to redistribute this software under certain conditions.",
        "\nSee <https://github.com/4jamesccraven/mkdev/blob/main/LICENSE> for more details."
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

    /// Alias for `mk imprint --interactive`
    #[arg(short, long)]
    pub interactive: bool,

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

impl Cli {
    /// Determine main logic based on the given arguments.
    pub fn dispatch(self, recipes: HashMap<String, Recipe>) -> Result<(), Error> {
        use crate::menus::editor;
        use crate::recipe::{build_recipes, delete_recipe, imprint_recipe, list_recipe};

        match self.command {
            Some(command) => match command {
                Commands::Evoke(sub_args) => build_recipes(sub_args, recipes),
                Commands::Imprint(sub_args) => imprint_recipe(sub_args, recipes),
                Commands::Delete(sub_args) => delete_recipe(sub_args, recipes),
                Commands::List(sub_args) => list_recipe(sub_args, recipes),
                Commands::Edit(sub_args) => editor(sub_args, recipes),
            },
            None if self.interactive => {
                let fake_args = Imprint {
                    interactive: true,
                    ..Default::default()
                };

                imprint_recipe(fake_args, recipes)
            }
            None => {
                // Print help and exit if no action is provided
                Cli::command().print_help().unwrap();
                Ok(())
            }
        }
    }
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
    /// Edit a recipe in-place.
    Edit(Edit),
}

#[derive(Parser, Clone, Debug)]
pub struct Evoke {
    /// The recipe(s) to build
    #[arg(add = ArgValueCompleter::new(recipe_completer))]
    pub recipes: Vec<String>,

    /// Evoke recipes in interactive mode
    #[arg(short, long)]
    pub interactive: bool,

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

#[derive(Parser, Debug, Default)]
pub struct Imprint {
    /// The name of the recipe to imprint.
    #[arg(default_value = "", required_unless_present = "interactive")]
    pub recipe: String,

    /// Create the recipe in interactive mode.
    #[arg(short, long)]
    pub interactive: bool,

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

    /// Disable default filters (e.g., .gitignore files)
    #[arg(long)]
    pub no_filter: bool,
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

    /// Hide description (note: only only applies default style)
    #[arg(long)]
    pub no_description: bool,
}

#[derive(Parser, Debug)]
pub struct Edit {
    /// The recipe to edit
    pub recipe: String,
}
