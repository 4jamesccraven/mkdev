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

//! Runtime manpage generation for mkdev and its configuration file.
use crate::cli::Cli;
use crate::error::{Context, Error};
use crate::fs;
use crate::man::MkdevMan;

use std::path::PathBuf;

use clap::CommandFactory;

/// Generates all of mkdev's man pages and saves them to './mkdev-man'.
pub fn hook(args: &Cli) -> Result<(), Error> {
    if args.man_page {
        let man_dir = PathBuf::from("./mkdev-man");
        fs::create_dir_all(&man_dir, Context::Man)?;

        // Try to write the man page for each of the following into './mkdev-man':
        // - The top-level command `mk`
        // - The config file
        // - Every subcommand
        [MkdevMan::command(), MkdevMan::config()]
            .into_iter()
            .chain(
                Cli::command()
                    .get_subcommands()
                    .map(|sc| MkdevMan::subcommand(sc.get_name())),
            )
            .try_for_each(|manpage| manpage.write_in(&man_dir))?;

        std::process::exit(0);
    }

    Ok(())
}
