use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(
    name = "mdev",
    version = "alpha 3.0.0",
    author = "James Craven <4jamesccraven@gmail.com>",
    about = "A command-line program that creates a development environment from user-defined config files.",
    subcommand_negates_reqs(true),
)]
pub struct Cli {
    #[arg(required = true)]
    pub recipe: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}


#[derive(Subcommand, Debug)]
pub enum Commands {
    Imprint {
        recipe: String,

        #[arg(short, long)]
        description: Option<String>,
    },
    Delete {
        recipe: String,
    }
}
