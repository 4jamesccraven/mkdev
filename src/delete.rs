use crate::cli::Delete;
use crate::mkdev_error::Error::{self, *};
use crate::recipe::Recipe;

use std::collections::HashMap;

/// Attempt to delete a config from the users directory, returning an error on failure
pub fn delete_recipe(args: Delete, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let to_delete = user_recipes.get(args.recipe.as_str());

    match to_delete {
        Some(recipe) => {
            let deleted_file = recipe.delete().map_err(|error| {
                Error::from_io(&format!("Unable to delete `{}`", recipe.name), &error)
            })?;

            println!("Deleted recipe at {}.", &deleted_file.display());

            Ok(())
        }
        None => Err(Invalid("recipe".into(), Some(vec![args.recipe]))),
    }
}
