use crate::recipe::Recipe;

use std::collections::HashMap;

/// List out a recipe or its contents, returning error messages on failure
pub fn list_recipe(
    recipe: Option<String>,
    user_recipes: &HashMap<String, Recipe>,
) -> Result<(), String> {
    match recipe {
        Some(recipe) => {
            let recipe = user_recipes.get(recipe.as_str()).ok_or_else(|| {
                format!("No such recipe \"{recipe}\". Run `mk list` to see valid recipes.")
            })?;

            println!("{}", recipe.display_contents());
        }
        None => {
            for recipe in user_recipes.values() {
                println!("{}\n", recipe);
            }
        }
    }

    Ok(())
}
