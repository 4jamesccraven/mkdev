use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(
    name = "mkdev",
    version = "alpha 3.0.0",
    author = "James Craven <4jamesccraven@gmail.com>",
    about = "A command-line program that creates a development environment from user-defined config files.",
    subcommand_negates_reqs(true),
)]
pub struct Cli {
    /// The recipe to construct
    #[arg(required = true)]
    pub recipe: Option<String>,

    /// Target directory for recipe
    pub dir_name: Option<String>,

    /// Alias for target-less `mk list`
    #[arg(long, short)]
    pub list: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}


#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a recipe by "imprinting" the contents
    /// of the current directory
    Imprint {
        /// The name of the recipe to imprint.
        /// NOTE: this action IS destructive and
        /// can overwrite existing recipes
        recipe: String,

        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete a recipe
    Delete {
        /// The recipe to delete
        recipe: String,
    },
    /// List recipes, or the contents of a specific one
    List {
        recipe: Option<String>
    }
}
