mod cli;
mod config;

use cli::Cli;

use clap::{Parser, Subcommand};

fn main() {
    let args = Cli::parse();

    let recipes = config::gather();

    if let Err(e) = recipes {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let recipes = recipes.unwrap();

    println!("{args:?}");
}
