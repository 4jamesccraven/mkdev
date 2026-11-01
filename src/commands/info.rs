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

//! Implementation of `mk info`.
//!
//! Displays man pages and other information.
use super::StoreCommand;

use crate::cli::SubcommandType;
use crate::error::Error;
use crate::man::MkdevMan;
use crate::recipe::RecipeStore;

use std::sync::OnceLock;

use clap::builder::PossibleValue;
use clap::{CommandFactory, Parser, ValueEnum};
use strum::VariantArray;

#[derive(Debug, Parser)]
pub struct Info {
    /// Documentation to display.
    pub item: Option<InfoItem>,
}

impl StoreCommand for Info {
    fn execute(self, store: RecipeStore) -> Result<(), Error> {
        info_command(self, store)
    }
}

pub fn info_command(args: Info, _: RecipeStore) -> Result<(), Error> {
    if let Some(item) = args.item {
        match item {
            InfoItem::Command(c) => MkdevMan::subcommand_discriminant(c).open(),
            InfoItem::Config => MkdevMan::config().open(),
            InfoItem::Aliases => {
                print!("{}", crate::presentation::format_aliases());
                Ok(())
            }
        }
    } else {
        // Fallback to avoid confusion with `mk help` alias.
        crate::cli::Cli::command().print_long_help().unwrap();
        Ok(())
    }
}

// ------------------------------------------------------------
// Info Items
// ------------------------------------------------------------

/// The possible information queries for the the `mk info` command.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InfoItem {
    /// One of the program's subcommands.
    Command(SubcommandType),
    /// The config file.
    Config,
    /// Lists aliases.
    Aliases,
}

impl ValueEnum for InfoItem {
    fn value_variants<'a>() -> &'a [Self] {
        static VARIANTS: OnceLock<Vec<InfoItem>> = OnceLock::new();

        VARIANTS.get_or_init(|| {
            SubcommandType::VARIANTS
                .iter()
                .cloned()
                .map(Self::Command)
                .chain([Self::Config, Self::Aliases])
                .collect()
        })
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::Command(command) => {
                let help = match command {
                    SubcommandType::Evoke => "Man page for 'mk evoke'",
                    SubcommandType::Imprint => "Man page for 'mk imprint'",
                    SubcommandType::Delete => "Man page for 'mk delete'",
                    SubcommandType::List => "Man page for 'mk list'",
                    SubcommandType::Alter => "Man page for 'mk alter'",
                    SubcommandType::Info => "Man page for 'mk info'",
                };

                command.to_possible_value().map(|pv| pv.help(help))
            }
            Self::Config => Some(PossibleValue::new("config").help("Man page for config file")),
            Self::Aliases => {
                Some(PossibleValue::new("aliases").help("Lists all aliased CLI values."))
            }
        }
    }
}
