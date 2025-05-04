use crate::cli::Evoke;
use crate::mkdev_error::Error::{self, *};
use crate::recipe::Recipe;
use crate::subs::Replacer;

use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;

/// Create all requested directory in the requested directories
pub fn build_recipes(args: Evoke, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let mut args = args;

    if args.recipes.is_empty() {
        return Err(NoneSpecified("recipes".into()));
    }

    let non_existant_recipes: Vec<String> = args
        .recipes
        .iter()
        .filter_map(|r| match user_recipes.contains_key(r) {
            false => {
                let r = r.to_string();
                Some(r)
            }
            true => None,
        })
        .collect();

    // There is an error if there are any non-existent recipes specified by the user
    if !non_existant_recipes.is_empty() {
        return Err(Invalid("recipe(s)".into(), Some(non_existant_recipes)));
    }

    if let None = &args.name {
        args.name = Some("NAME".into());
    }

    let re = Replacer::new();

    // Build to the cwd, or a directory specified by the user
    let dir = match &args.dir_name {
        Some(dir) => PathBuf::from(dir),
        None => current_dir().map_err(|why| Error::from_io("unable to get cwd", &why))?,
    };

    let extra_args = args.clone();
    args.recipes.iter().try_for_each(|r| {
        let recipe = user_recipes
            .get(r)
            .expect("Invalid recipes should have been filtered out.");

        Recipe::build(&dir, &recipe.contents, &extra_args, &re).map_err(|why| {
            let context = format!("Unable to write `{}` to `{}`", recipe.name, dir.display());
            Error::from_io(&context, &why)
        })
    })?;

    Ok(())
}
