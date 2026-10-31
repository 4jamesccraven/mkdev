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
use super::style::mkdev_style;

#[expect(unused)]
use crate::commands::{self, *};
use crate::config::Config;
use crate::error::Error;
use crate::man::MkdevMan;
use crate::recipe::RecipeStore;
use crate::warning;

use std::convert::identity;
use std::path::PathBuf;

use clap::builder::FalseyValueParser;
use clap::{
    CommandFactory, Parser, Subcommand, ValueEnum, crate_authors, crate_description, crate_version,
};
use rust_i18n::t;
use strum::{EnumDiscriminants, VariantArray};

/// Command Line Interface for Mkdev
#[derive(Parser, Debug)]
#[command(
    // Metadata
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
    // CLI Config
    styles = mkdev_style(),
    max_term_width = 90,
    disable_help_subcommand = true,
)]
pub struct Cli {
    /// Subcommand.
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Override the config path.
    #[arg(short, long, env = "MKDEV_CONFIG")]
    pub config: Option<PathBuf>,

    /// Alias for 'mk imprint --interactive'.
    #[arg(short, long)]
    pub interactive: bool,

    /// Write man pages.
    ///
    /// Man pages are written to './mkdev-man'
    #[arg(long, env = "MANPAGE", value_parser = FalseyValueParser::new())]
    pub man_page: bool,

    /// Print the default config.
    #[arg(short, long)]
    pub gen_config: bool,

    /// Print the current config.
    ///
    /// This serializes the config as evaluated; i.e., the output of this command is not directly
    /// equivalent to the contents of the config file itself.
    #[arg(short, long)]
    pub print_config: bool,

    /// Print license (see SPDX License Identifier with '-L').
    #[arg(long = "license")]
    pub print_license: bool,
    #[arg(short = 'L', hide = true)]
    pub print_license_identifier: bool,
}

impl Cli {
    /// Execute the parsed subcommand.
    pub fn execute(self, store: RecipeStore) -> Result<(), Error> {
        use crate::commands::StoreCommand;

        let command_present = self.command.is_some();
        let skips_commands = [
            self.man_page,
            self.gen_config,
            self.print_config,
            self.print_license,
            self.print_license_identifier,
        ];

        if command_present && skips_commands.into_iter().any(identity) {
            warning!("{}", t!("warnings.subcommand_suppressed"));
        }

        match self.command {
            // Flags that suppress commands (hooks)
            _ if self.man_page => MkdevMan::write_all("./mkdev-man"),
            _ if self.print_config => Config::get()?.print(),
            _ if self.gen_config => Config::default().print(),
            _ if self.print_license => {
                println!("{}", include_str!("../../LICENSE"));
                Ok(())
            }
            _ if self.print_license_identifier => {
                // Parsed from Cargo.toml at compile-time.
                println!("{}", env!("LICENSE_IDENT"));
                Ok(())
            }
            // Normal business logic
            Some(command) => match command {
                Commands::Evoke(args) => args.execute(store),
                Commands::Imprint(args) => args.execute(store),
                Commands::Delete(args) => args.execute(store),
                Commands::List(args) => args.execute(store),
                Commands::Alter(args) => args.execute(store),
                Commands::Info(args) => args.execute(store),
            },
            // Handle interactive imprint alias
            None if self.interactive => Imprint {
                interactive: true,
                ..Default::default()
            }
            .execute(store),
            // Fallback if no arguments are received whatsoever.
            None => {
                // Print help and exit if no action is provided
                Cli::command().print_help().unwrap();
                Ok(())
            }
        }
    }
}

/// CLI adaptor for each of the commands in [`commands`].
///
/// ## About `about`
/// Each about string must be the same length so that the alias boxes are aligned. This makes it
/// easier for the user to quickly parse what each subcommand is for.
#[derive(Subcommand, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(ValueEnum, VariantArray, strum::Display))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(SubcommandType))]
#[strum_discriminants(doc = "The name of a subcommand")]
pub enum Commands {
    #[command(
        about = "Build recipes   ",
        long_about = "Evocation is the process of instantiating a recipe. Recipe contents are \
            replicated in the current directory (by default), and substitutions are applied. ",
        visible_aliases = ["e", "build"],
        // Easter eggs
        aliases = ["conjure", "invoke", "summon"])
    ]
    Evoke(Evoke),

    #[command(
        about = "Define recipes  ",
        long_about = "Imprinting is the process of defining a recipe. `mkdev` traverses the \
            current directory to collect files and directories, and serializes them in the \
            mkdev store. ",
        visible_aliases = ["clone"])
    ]
    Imprint(Imprint),

    #[command(
        about = "Delete recipes  ",
        long_about = "Delete any number of recipes by ID or by namespace.",
        visible_aliases = ["remove", "rm"])
    ]
    Delete(Delete),

    #[command(
        about = "List recipes    ",
        long_about = "Display stored recipes in various formats. By default, all recipes are \
            listed with a short summary, grouped by namespace. Listing a single recipe shows its \
            contents instead.",
        visible_aliases = ["ls", "show"])
    ]
    List(List),

    #[command(
        about = "Edit recipes    ",
        long_about = "Launches an interactive editor for the provided recipe. File contents are \
            edited in the user's default $EDITOR.",
        visible_aliases = ["edit"])
    ]
    Alter(Alter),

    #[command(
        about = "Help & reference",
        long_about = "Display man pages and other documentation. The man program is required for \
            this functionality.",
        visible_aliases = ["help"],
    )]
    Info(Info),
}
