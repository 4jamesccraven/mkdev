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

//! Implementation of `mk store`.
use super::MkdevCommand;

use crate::{error::Error, store::RecipeStore};

use clap::{Parser, Subcommand};

pub mod export;
pub mod import;
pub mod scry;

#[derive(Parser, Debug)]
#[command(disable_help_subcommand = true)]
pub struct Store {
    /// Command to run
    #[command(subcommand)]
    pub command: StoreSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum StoreSubcommand {
    #[command(
        about = "Recipe file details",
        long_about = "List filesystem details and basic metadata about a recipe.",
        visible_aliases = ["stat"],
    )]
    Scry(scry::Scry),

    #[command(
        about = "Backup the store   ",
        long_about = "Make backups of the store or a specific store index.",
        visible_aliases = ["ex"],
    )]
    Export(export::Export),

    #[command(
        about = "Load store backups ",
        long_about = "Restore previously created backups of the store.",
        visible_aliases = ["im"],
    )]
    Import(import::Import),
}

impl MkdevCommand for Store {
    fn execute(self, store: RecipeStore) -> Result<(), Error> {
        store_command(self, store)
    }
}

pub fn store_command(args: Store, store: RecipeStore) -> Result<(), Error> {
    match args.command {
        StoreSubcommand::Scry(args) => args.execute(store),
        StoreSubcommand::Export(args) => args.execute(store),
        StoreSubcommand::Import(args) => args.execute(store),
    }
}
