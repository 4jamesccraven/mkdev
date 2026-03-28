//! mkdev's core library. Defines the recipe schema and provides tools for working with them.
mod delete;
mod evoke;
mod imprint;
mod lang;
mod list;
mod version;

pub use delete::*;
pub use evoke::*;
pub use imprint::*;
pub use lang::Language;
pub use list::*;

use version::*;

use crate::config::Config;
use crate::content::RecipeItem;
use crate::fs_wrappers;
use crate::mkdev_error::{Context, Error};
use crate::warning;

use std::collections::HashMap;
use std::path::PathBuf;

use dirs::data_dir;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

/// A mkdev recipe (v2).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipe {
    /// A unique identifier for the recipe.
    ///
    /// Determines both the name of the file the recipe is stored in as well as how mkdev will
    /// display information about it.
    pub name: String,
    /// A description of the recipe (Optional)
    #[serde(default = "String::new")]
    pub description: String,
    /// The programming languages (or file formats) found in the recipe's contents.
    pub languages: Vec<Language>,
    /// The contents the recipe holds.
    pub contents: Vec<RecipeItem>,
}

impl Recipe {
    /// Gathers all recipes from the user directory.
    ///
    /// Only files with the .toml extension are checked. An invalid recipe gives a warning.
    pub fn gather() -> Result<HashMap<String, Recipe>, Error> {
        let data_dir = recipe_dir()?;
        let files = fs_wrappers::read_dir(data_dir, Context::Gather)?;
        let mut recipes: Vec<Recipe> = Vec::new();

        for file in files {
            let path = file.map_err(|e| crate::borked!(e)).unwrap().path();

            if path.extension() == Some(std::ffi::OsStr::new("toml")) && path.is_file() {
                let file_contents = fs_wrappers::read_to_string(&path, Context::Gather)?;
                let recipe = deserialise_recipe(&file_contents);

                match recipe {
                    Some(recipe) => {
                        recipes.push(recipe);
                    }
                    None => {
                        warning!("{}", t!("warnings.invalid_recipe", path => path.display()));
                    }
                }
            }
        }

        let recipes = recipes
            .iter()
            .map(|r| (r.name.clone(), r.to_owned()))
            .collect();

        Ok(recipes)
    }
}

/// Gets the user's preferred data dir, or uses the default XDG_DATA_DIR.
pub fn recipe_dir() -> Result<PathBuf, Error> {
    let cfg = Config::get()?;

    let data_dir = match &cfg.recipe_dir {
        Some(dir) => dir.clone(),
        None => {
            let mut temp = data_dir().expect("$HOME is not set; cannot determine data directory.");
            temp.push("mkdev");
            temp
        }
    };

    if !data_dir.is_dir() {
        fs_wrappers::create_dir_all(&data_dir, Context::Gather)?;
    }

    Ok(data_dir)
}
