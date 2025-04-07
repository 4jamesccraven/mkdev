use crate::cli::Cli;
use crate::recipe::Recipe;
use crate::subs::Replacer;

use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;

/// Create all requested directory in the requested directories
pub fn build_recipes(args: Cli, user_recipes: HashMap<String, Recipe>) -> Result<(), String> {
    let rec_args = args
        .recipes
        .expect("The argument parser should catch this if it's none.");

    let non_existant_recipes: Vec<String> = rec_args
        .iter()
        .filter_map(|r| match user_recipes.contains_key(r) {
            false => Some(r),
            true => None,
        })
        .map(|r| format!("No such recipe \"{r}\"."))
        .collect();

    let is_err = !non_existant_recipes.is_empty();

    if is_err {
        let message = format!("Invalid recipes:\n{}", non_existant_recipes.join("\n"));
        return Err(message);
    }

    let re = Replacer::new();

    let dir = match &args.dir_name {
        Some(dir) => PathBuf::from(dir),
        None => current_dir().map_err(|error| format!("Unable to get cwd: {error:?}"))?,
    };

    rec_args.iter().try_for_each(|r| {
        let recipe = user_recipes
            .get(r)
            .expect("Invalid recipes should have been filtered out.");

        Recipe::build(&dir, &recipe.contents, args.verbose, &re).map_err(|error| {
            format!(
                "Unable to write `{}` to `{}`: {error:?}",
                recipe.name,
                dir.display()
            )
        })
    })?;

    Ok(())
}
