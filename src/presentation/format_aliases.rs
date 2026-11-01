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

//! A simple formatter that displays aliases for CLI options and commands.
use crate::cli::Cli;

use std::fmt::Write;

use clap::CommandFactory;
use colored::Colorize;

pub fn format_aliases() -> String {
    let command = Cli::command();
    let mut out = String::new();

    for subcommand in command.get_subcommands() {
        // Write the subcommand as a header.
        _ = writeln!(&mut out, "{}:", subcommand.get_name().green());

        // Render aliases for the subcommand itself
        let aliases = subcommand.get_all_aliases().collect::<Vec<_>>().join(", ");
        if !aliases.is_empty() {
            _ = writeln!(&mut out, "  {}: {aliases}", "aliases".yellow());
        }

        // Attempt to render aliases for its args.
        for arg in subcommand.get_arguments() {
            let long_aliases = arg.get_all_aliases();
            let short_aliases = arg.get_all_short_aliases();
            let has_aliases = long_aliases.is_some() || short_aliases.is_some();

            if has_aliases {
                let header = match (arg.get_short(), arg.get_long()) {
                    (Some(s), None) => format!("-{s}"),
                    (None, Some(l)) => format!("--{l}"),
                    (Some(s), Some(l)) => format!("-{s},--{l}"),
                    _ => unreachable!("Positional arg can't have alias"),
                }
                .yellow();

                _ = write!(&mut out, "    {header}: ");
            }

            if let Some(shorts) = short_aliases {
                _ = write!(
                    &mut out,
                    "{}{}",
                    shorts
                        .into_iter()
                        .map(|s| format!("-{s}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    if long_aliases.is_some() { ", " } else { "" }
                );
            }

            if let Some(longs) = long_aliases {
                _ = write!(
                    &mut out,
                    "{}",
                    longs
                        .into_iter()
                        .map(|l| format!("--{l}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            }

            if has_aliases {
                _ = writeln!(&mut out);
            }
        }

        _ = writeln!(&mut out);
    }

    out
}
