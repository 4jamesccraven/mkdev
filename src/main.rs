mod cli;
mod config;
mod content;
mod hooks;
mod mkdev_error;
mod output_type;
mod recipe;
mod recipe_completer;
mod subs;

use cli::{Cli, Commands::*};
use hooks::hooks;
use mkdev_error::{Error::*, ResultExt};
use recipe::Recipe;
use recipe::{build_recipes, delete_recipe, imprint_recipe, list_recipe};

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;

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

/// Dispatcher for various actions
fn try_get_status(args: Cli) -> Result<(), mkdev_error::Error> {
    // Arguments that cause an exit before subcommand logic
    hooks(&args);

    let user_recipes = Recipe::gather().context("unable to load recipes")?;

    match args.command {
        Some(command) => match command {
            Evoke(sub_args) => build_recipes(sub_args, user_recipes),
            Imprint(sub_args) => imprint_recipe(sub_args, user_recipes),
            Delete(sub_args) => delete_recipe(sub_args, user_recipes),
            List(sub_args) => list_recipe(sub_args, user_recipes),
        },
        None => Err(NoneSpecified("action".into())),
    }
}
