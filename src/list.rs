use crate::mkdev_error::Error::{self, *};
use crate::output_type::OutputType::{self, *};
use crate::recipe::Recipe;

use std::collections::HashMap;

use ser_nix;
use serde_json;
use toml;

/// List out a recipe or its contents, returning error messages on failure
pub fn list_recipe(
    recipe: Option<String>,
    output_type: Option<OutputType>,
    user_recipes: HashMap<String, Recipe>,
) -> Result<(), Error> {
    let output_type = match output_type {
        Some(output_type) => output_type,
        None => OutputType::Default,
    };

    match recipe {
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
