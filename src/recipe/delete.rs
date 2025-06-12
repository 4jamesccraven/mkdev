use super::{recipe_dir, Recipe};
use crate::cli::Delete;
use crate::mkdev_error::{
    Error::{self, *},
    ResultExt,
};

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Attempt to delete a config from the users directory, returning an error on failure
pub fn delete_recipe(args: Delete, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let to_delete = user_recipes.get(args.recipe.as_str());

    match to_delete {
        Some(recipe) => {
            let deleted_file = recipe
                .delete()
                .context(&format!("unable to delete `{}`", recipe.name))?;

            println!("Deleted recipe at {}.", &deleted_file.display());

            Ok(())
        }
        None => Err(Invalid("recipe".into(), Some(vec![args.recipe]))),
    }
}

impl Recipe {
    /// Delete the recipe by deleting its serialised self
    pub fn delete(&self) -> io::Result<PathBuf> {
        let mut data_dir = recipe_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs::remove_file(&data_dir)?;

        Ok(data_dir)
    }
}
