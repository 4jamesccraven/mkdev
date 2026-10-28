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

//! Top-level parser for all of mkdev's commands.
#[expect(unused)]
use crate::commands::{self, *};
use crate::error::Error;
use crate::recipe::RecipeStore;

use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand, crate_authors, crate_description, crate_version};

/// Command Line Interface for Mkdev
#[derive(Parser, Debug)]
#[command(
    name = "mkdev",
    version = crate_version!(),
    long_version = concat!(
        crate_version!(), " --- ", crate_description!(),
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
    /// Subcommand.
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Override the config path.
    #[arg(short, long, env = "MKDEV_CONFIG")]
    pub config: Option<PathBuf>,

    /// Alias for `mk imprint --interactive`.
    #[arg(short, long)]
    pub interactive: bool,

    /// Write man pages.
    ///
    /// Man pages are written to `./mkdev-man`
    #[arg(long, env = "MANPAGE")]
    pub man_page: bool,

    /// Print the default config.
    #[arg(short, long)]
    pub gen_config: bool,

    /// Print the current config.
    #[arg(short, long)]
    pub print_config: bool,
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

/// CLI adaptor for each of the commands in [`commands`].
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// | Build recipes  |
    ///
    /// Evocation is the process of instantiating a recipe. Recipe contents are replicated in the
    /// current directory (by default), and substitutions are applied.
    #[command(visible_aliases = ["build"], aliases = [ "conjure", "invoke", "summon"])]
    Evoke(Evoke),
    /// | Define recipes |
    ///
    /// Imprinting is the process of defining a recipe. `mkdev` traverses the current directory to
    /// collect files and directories, and serializes them in the mkdev store.
    #[command(visible_aliases = ["clone"])]
    Imprint(Imprint),
    /// | Delete recipes |
    ///
    /// Delete any number of recipes by ID or by namespace.
    #[command(visible_aliases = ["remove", "rm"])]
    Delete(Delete),
    /// | List recipes   |
    ///
    /// Display stored recipes in various formats. By default, all recipes are listed with a short
    /// summary, grouped by namespace. Listing a single recipe shows its contents instead.
    #[command(visible_aliases = ["show", "ls"])]
    List(List),
    /// | Edit recipes   |
    ///
    /// Launches an interactive editor for the provided recipe. File contents are edited in the
    /// user's default $EDITOR.
    #[command(visible_aliases = ["edit"])]
    Alter(Alter),
}
