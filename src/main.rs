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

use cli::{Cli, Commands::*};
use hooks::hooks;
use menus::editor;
use recipe::Recipe;
use recipe::{build_recipes, delete_recipe, imprint_recipe, list_recipe};

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    // Produce completion scripts using clap_complete...
    // note: this cannot be included in hooks because it must happen before parsing the command
    // line.
    CompleteEnv::with_factory(Cli::command).complete();
    // ... or try to do mkdev's business logic
    let status = try_get_status(Cli::parse());

    // Inform user of error, then exit with fail code
    if let Err(why) = status {
        die!("{why}");
    }
}

fn try_get_status(args: Cli) -> Result<(), mkdev_error::Error> {
    // Handle arguments that are tangential or mutually exclusive with general
    // recipe logic.
    hooks(&args)?;

    let user_recipes = Recipe::gather()?;

    match args.command {
        // TODO: can probably be simplified with a trait or smth.
        Some(command) => match command {
            Evoke(sub_args) => build_recipes(sub_args, user_recipes),
            Imprint(sub_args) => imprint_recipe(sub_args, user_recipes),
            Delete(sub_args) => delete_recipe(sub_args, user_recipes),
            List(sub_args) => list_recipe(sub_args, user_recipes),
            Edit(sub_args) => editor(sub_args, user_recipes),
        },
        None if args.interactive => {
            let fake_args = cli::Imprint {
                interactive: true,
                ..Default::default()
            };

            imprint_recipe(fake_args, user_recipes)
        }
        None => {
            // Print help and exit if no action is provided
            Cli::command().print_help().unwrap();
            Ok(())
        }
    }
}
