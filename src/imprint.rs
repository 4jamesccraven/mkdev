use crate::recipe::Recipe;

/// Atttempts to call recipe's imprint and save methods, returning an error message
/// on failure
pub fn imprint_recipe(recipe: String, description: Option<String>) -> Result<(), String> {
    let new = Recipe::imprint(recipe, description).map_err(|error| {
        format!("Unable to read current_working directory for the recipe: {error:?}")
    })?;
    let save_location = new
        .save()
        .map_err(|error| format!("Unable to save instantiated recipe: {error:?}"))?;

    println!("Recipe saved successfully to {}.", &save_location.display());

    Ok(())
}
