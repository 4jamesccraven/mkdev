use mkdev_cli::output_type::OutputType::{self, *};
use mkdev_recipe::recipe::Recipe;

use std::collections::HashMap;

use serde_json;
use toml;

/// List out a recipe or its contents, returning error messages on failure
pub fn list_recipe(
    recipe: Option<String>,
    output_type: Option<OutputType>,
    user_recipes: &HashMap<String, Recipe>,
) -> Result<(), String> {
    let output_type = match output_type {
        Some(output_type) => output_type,
        None => OutputType::Default,
    };

    match recipe {
        Some(recipe) => {
            let recipe = user_recipes.get(recipe.as_str()).ok_or_else(|| {
                format!("No such recipe \"{recipe}\". Run `mk list` to see valid recipes.")
            })?;

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
        _ => unreachable!(),
    }
}
