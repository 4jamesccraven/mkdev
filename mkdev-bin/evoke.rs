use mkdev_recipe::recipe::Recipe;
use mkdev_recipe::subs::Replacer;

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
    if recipes.is_empty() {
        return Err("No recipes specified.".to_string());
    }

    let non_existant_recipes: Vec<String> = recipes
        .iter()
        .filter_map(|r| match user_recipes.contains_key(r) {
            false => Some(r),
            true => None,
        })
        .map(|r| r.to_string())
        .collect();

    // There is an error if there are any non-existent recipes specified by the user
    if !non_existant_recipes.is_empty() {
        let message = format!("Invalid recipe(s):\n{}", non_existant_recipes.join("\n"));
        return Err(message);
    }

    let re = Replacer::new();

    // Build to the cwd, or a directory specified by the user
    let dir = match dir_name {
        Some(dir) => PathBuf::from(dir),
        None => current_dir().map_err(|why| format!("Unable to get cwd: {why}"))?,
    };

    recipes.iter().try_for_each(|r| {
        let recipe = user_recipes
            .get(r)
            .expect("Invalid recipes should have been filtered out.");

        Recipe::build(&dir, &recipe.contents, verbose, &re).map_err(|why| {
            format!(
                "Unable to write `{}` to `{}`: {why}",
                recipe.name,
                dir.display()
            )
        })
    })?;

    Ok(())
}
