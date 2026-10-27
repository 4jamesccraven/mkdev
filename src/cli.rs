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
use crate::commands::*;
use crate::error::Error;
use crate::recipe::RecipeStore;

pub use crate::output_type::OutputType;
pub use crate::recipe_completer::recipe_completer;

use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand, crate_authors, crate_description, crate_version};

/// Command Line Interface for Mkdev
#[derive(Parser, Debug)]
#[command(
    name = "mk",
    version = crate_version!(),
    long_version = concat!(
        crate_version!(), " — ", crate_description!(),
        "\n© 2026 ", crate_authors!(), ".",
        "\nThis program is free software and comes with ABSOLUTELY NO WARRANTY.",
        "\nYou are welcome to redistribute this software under certain conditions.",
        "\nSee <https://github.com/4jamesccraven/mkdev/blob/main/LICENSE> for more details."
    ),
    author = crate_authors!(),
    about = crate_description!(),
    disable_help_subcommand = true,
)]
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
    /// Execute the parsed subcommand.
    pub fn execute(self, store: RecipeStore) -> Result<(), Error> {
        use crate::commands::StoreCommand;

        match self.command {
            Some(command) => match command {
                Commands::Evoke(args) => args.execute(store),
                Commands::Imprint(args) => args.execute(store),
                Commands::Delete(args) => args.execute(store),
                Commands::List(args) => args.execute(store),
                Commands::Alter(args) => args.execute(store),
            },
            // Handle `mk -i` alias
            None if self.interactive => Imprint {
                interactive: true,
                ..Default::default()
            }
            .execute(store),
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
    #[command(aliases = ["remove", "rm"])]
    Delete(Delete),
    /// List recipes, or the contents of a specific one [Alias: show]
    #[command(aliases = ["show"])]
    List(List),
    /// Edit a recipe in-place. [Alias: edit]
    #[command(aliases = ["edit"])]
    Alter(Alter),
}
