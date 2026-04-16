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
mod cli;
mod config;
mod content;
mod display;
mod fs_wrappers;
mod hooks;
mod menus;
mod mkdev_error;
mod output_type;
mod recipe;
mod recipe_completer;
mod replacer;

use cli::Cli;
use hooks::hooks;
use recipe::Recipe;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    // Produce completion scripts using clap_complete.
    // note: this cannot be included in hooks because it must happen before parsing the command
    // line.
    CompleteEnv::with_factory(Cli::command).complete();

    // Parse the command line, run hooks, and then dispatch main logic.
    let args = Cli::parse();
    let status = hooks(&args)
        .and_then(|_| Recipe::gather())
        .and_then(|recipes| args.dispatch(recipes));

    // Inform user of error, then exit with fail code
    if let Err(why) = status {
        die!("{why}");
    }
}
