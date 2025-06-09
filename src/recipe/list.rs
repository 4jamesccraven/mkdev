use super::Recipe;
use crate::cli::List;
use crate::content::Content;
use crate::mkdev_error::Error::{self, *};
use crate::output_type::OutputType::{self, *};

use std::collections::HashMap;

use colored::Colorize;
use ser_nix;
use serde_json;
use toml;

/// List out a recipe or its contents, returning error messages on failure
pub fn list_recipe(args: List, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let output_type = match args.r#type {
        Some(output_type) => output_type,
        None => OutputType::Default,
    };

    match args.recipe {
        Some(recipe) => {
            let recipe = user_recipes
                .get(recipe.as_str())
                .ok_or_else(|| Invalid("recipe".into(), Some(vec![recipe])))?;

            display_one(recipe, output_type);
        }
        None => {
            let mut recipes: Vec<_> = user_recipes.values().collect();
            recipes.sort_by(|a, b| a.name.cmp(&b.name));

            display_all(recipes, output_type);
        }
    }

    Ok(())
}

fn display_all(recipes: Vec<&Recipe>, output_type: OutputType) {
    if let TOML = output_type {
        eprintln!(concat!(
            "Option \"TOML\" invalid for displaying all recipes. ",
            "Select a single recipe to if you wish to use this format."
        ));

        return;
    }

    match output_type {
        Default => recipes.iter().for_each(|r| println!("{}\n", r)),
        #[rustfmt::skip]
        Debug => recipes.iter().for_each(|r| { dbg!(r); }),
        Plain => recipes.iter().for_each(|r| println!("{}", r.name)),
        JSON => println!(
            "{}",
            serde_json::to_string_pretty(&recipes)
                .expect("Recipes are built with serde, and should unwrap")
        ),
        Nix => {
            let output = ser_nix::to_string(&recipes);

            match output {
                Ok(r) => println!("{r}"),
                Err(why) => eprintln!("error: {why}"),
            }
        }
        _ => unreachable!(),
    }
}

fn display_one(recipe: &Recipe, output_type: OutputType) {
    match output_type {
        Default => print!("{}", recipe.display_contents()),
        Debug => _ = dbg!(recipe),
        Plain => print!("{}", recipe.display_contents_plain()),
        JSON => println!(
            "{}",
            serde_json::to_string_pretty(recipe)
                .expect("Recipes are built with serde, and should unwrap")
        ),
        TOML => println!(
            "{}",
            toml::to_string_pretty(recipe)
                .expect("Recipes are built with serde, and should unwrap")
        ),
        Nix => {
            let output = ser_nix::to_string(&recipe);

            match output {
                Ok(r) => println!("{r}"),
                Err(why) => eprintln!("error: {why}"),
            }
        }
    }
}

impl Recipe {
    /// Display contents of `tree` with default style
    pub fn display_contents(&self) -> String {
        let mut out = format!("{}\n", self.name.bold().blue());
        let mut iter = self.contents.iter().peekable();

        while let Some(obj) = iter.next() {
            let next = obj.produce_tree_string("".into(), iter.peek().is_none());
            out.push_str(&next);
        }

        out
    }

    /// Display all file names associated with the recipe
    pub fn display_contents_plain(&self) -> String {
        let mut out = String::new();

        for obj in &self.contents {
            match obj {
                Content::File(file) => {
                    let filname = format!("{}\n", file.name);
                    out.push_str(&filname);
                }
                Content::Directory(dir) => {
                    let dir_contents = format!("{}", dir.produce_file_names());
                    out.push_str(&dir_contents);
                }
            }
        }

        out
    }
}
