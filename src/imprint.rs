use crate::recipe::Recipe;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use ser_nix;

/// Atttempts to call recipe's imprint and save methods, returning an error message
/// on failure
pub fn imprint_recipe(
    recipe: String,
    description: Option<String>,
    suppress_warnings: bool,
    to_nix: Option<PathBuf>,
    user_recipes: HashMap<String, Recipe>,
) -> Result<(), String> {
    let new = Recipe::imprint(recipe, description).map_err(|error| {
        format!("Unable to read current_working directory for the recipe: {error}")
    })?;

    if let Some(path) = to_nix {
        let nix_expression = match ser_nix::to_string(&new) {
            Ok(expr) => expr,
            Err(why) => {
                return Err(format!("unable to serialise recipe to nix: {why}"));
            }
        };

        fs::write(path, nix_expression)
            .map_err(|why| format!("unable to write to output file: {why}"))?;

        return Ok(());
    }

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
