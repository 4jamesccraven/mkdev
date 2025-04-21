use clap::{crate_authors, crate_description, crate_version, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "mkdev",
    version = crate_version!(),
    long_version = concat!(
        crate_version!(), " — ", crate_description!(), "\n",
        "© 2025 ", crate_authors!(), ". Licensed under MIT License — see LICENSE for details."
    ),
    author = crate_authors!(),
    about = crate_description!(),
    subcommand_negates_reqs(true)
)]
pub struct Cli {
    /// The recipe to construct
    #[arg(required = true)]
    pub recipes: Option<Vec<String>>,

    /// Target directory for recipe
    #[arg(last = true)]
    pub dir_name: Option<String>,

    /// Prints the name of each file on creation
    #[arg(short, long)]
    pub verbose: bool,

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
        /// Description to be associated with recipe
        description: Option<String>,
    },
    /// Delete a recipe
    Delete {
        /// The recipe to delete
        recipe: String,
    },
    /// List recipes, or the contents of a specific one
    List { recipe: Option<String> },
}
