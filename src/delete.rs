use crate::recipe::Recipe;

use std::collections::HashMap;

/// Attempt to delete a config from the users directory, returning an error on failure
pub fn delete_recipe(recipe: String, user_recipes: &HashMap<String, Recipe>) -> Result<(), String> {
    let to_delete = user_recipes.get(recipe.as_str());

    match to_delete {
        Some(recipe) => recipe
            .delete()
            .map_err(|error| format!("Unable to delete `{}`: {error:?}", recipe.name)),
        None => Err(String::from("No such recipe \"{recipe}\".")),
    }
}
