use crate::recipe::Recipe;
use crate::subs::Replacer;

use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;

/// Create all requested directory in the requested directories
pub fn build_recipes(
    recipes: Vec<String>,
    dir_name: Option<String>,
    verbose: bool,
    user_recipes: HashMap<String, Recipe>,
) -> Result<(), String> {
    let non_existant_recipes: Vec<String> = recipes
        .iter()
        .filter_map(|r| match user_recipes.contains_key(r) {
            false => Some(r),
            true => None,
        })
        .map(|r| format!("No such recipe \"{r}\"."))
        .collect();

    // There is an error if there are any non-existent recipes specified by the user
    let is_err = !non_existant_recipes.is_empty();

    if is_err {
        let message = format!("Invalid recipes:\n{}", non_existant_recipes.join("\n"));
        return Err(message);
    }

    let re = Replacer::new();

    // Build to the cwd, or a directory specified by the user
    let dir = match dir_name {
        Some(dir) => PathBuf::from(dir),
        None => current_dir().map_err(|error| format!("Unable to get cwd: {error:?}"))?,
    };

    recipes.iter().try_for_each(|r| {
        let recipe = user_recipes
            .get(r)
            .expect("Invalid recipes should have been filtered out.");

        Recipe::build(&dir, &recipe.contents, verbose, &re).map_err(|error| {
            format!(
                "Unable to write `{}` to `{}`: {error:?}",
                recipe.name,
                dir.display()
            )
        })
    })?;

    Ok(())
}
