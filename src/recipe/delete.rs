//! Implementation of `mk delete`.
//!
//! Used to delete recipes from their default location.
use super::{Recipe, recipe_dir};
use crate::cli::Delete;
use crate::mkdev_error::Context;
use crate::mkdev_error::{
    Error::{self, *},
    Subject,
};

use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use rust_i18n::t;

/// Deletes a recipe based on command line arguments.
pub fn delete_recipe(args: Delete, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let to_delete = user_recipes.get(args.recipe.as_str());

    match to_delete {
        Some(recipe) => {
            let deleted_file = recipe.delete()?;

            println!(
                "{}",
                t!("recipes.delete_msg", path => &deleted_file.display())
            );

            Ok(())
        }
        None => Err(Invalid {
            subject: Subject::Recipe,
            examples: Some(vec![args.recipe]),
        }),
    }
}

impl Recipe {
    /// Delete the recipe by deleting its serialised self
    pub fn delete(&self) -> Result<PathBuf, Error> {
        let mut data_dir = recipe_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs::remove_file(&data_dir).map_err(|e| match e.kind() {
            ErrorKind::PermissionDenied => Error::FsDenied {
                which: data_dir.clone(),
                context: Context::Delete,
            },
            _ => crate::borked!(e),
        })?;

        Ok(data_dir)
    }
}
