use crate::recipe::Recipe;

use std::collections::HashMap;

/// Atttempts to call recipe's imprint and save methods, returning an error message
/// on failure
pub fn imprint_recipe(
    recipe: String,
    description: Option<String>,
    suppress_warnings: bool,
    user_recipes: HashMap<String, Recipe>,
) -> Result<(), String> {
    let new = Recipe::imprint(recipe, description).map_err(|error| {
        format!("Unable to read current_working directory for the recipe: {error}")
    })?;

    let destructive = user_recipes.iter().any(|(recipe, _)| recipe == &new.name);

    if destructive && !suppress_warnings {
        return Err(format!("'{}' already exists.", new.name));
    }

    let save_location = new
        .save()
        .map_err(|error| format!("Unable to save instantiated recipe: {error}"))?;

    println!("{}", &save_location.display());

    Ok(())
}
