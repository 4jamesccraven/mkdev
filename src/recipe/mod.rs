mod delete;
mod evoke;
mod imprint;
mod lang;
mod list;
mod version;

use crate::warning;

pub use delete::*;
pub use evoke::*;
pub use imprint::*;
pub use list::*;

use version::*;

use lang::Language;

use crate::config::Config;
use crate::content::RecipeItem;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use colored::Colorize;
use dirs::data_dir;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipe {
    pub name: String,
    #[serde(default = "String::new")]
    pub description: String,
    pub languages: Vec<Language>,
    pub contents: Vec<RecipeItem>,
}

impl Recipe {
    /// Attempt to find all user defined recipes
    pub fn gather() -> io::Result<HashMap<String, Recipe>> {
        let data_dir = recipe_dir()?;
        let files = fs::read_dir(data_dir)?;
        let mut recipes: Vec<Recipe> = Vec::new();

        for file in files {
            let path = file?.path();

            if path.extension() == Some(std::ffi::OsStr::new("toml")) && path.is_file() {
                let file_contents = fs::read_to_string(&path)?;
                let recipe = deserialise_recipe(&file_contents);

                match recipe {
                    Some(recipe) => {
                        recipes.push(recipe);
                    }
                    None => {
                        warning!("{} is not a valid recipe.", path.display());
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

    /// Shows a summary of the recipe
    fn display_summary(&self, show_description: bool) -> String {
        let lang_string = self
            .languages
            .iter()
            .map(|l| format!("{l}"))
            .collect::<Vec<_>>()
            .join(" ");

        let desc = &format!("\n  {}", self.description);
        format!(
            "{} ( {} ){}",
            self.name.bold(),
            lang_string,
            if show_description { desc } else { "" }
        )
    }
}

/// Get the user's preferred data dir, or use the default XDG_DATA_DIR
pub fn recipe_dir() -> io::Result<PathBuf> {
    let cfg = match Config::get() {
        Ok(cfg) => cfg,
        Err(why) => {
            let err = io::Error::other(why);
            return Err(err);
        }
    };

    let err = io::Error::other("Error getting data directory");
    let data_dir = match &cfg.recipe_dir {
        Some(dir) => dir.clone(),
        None => {
            let mut temp = data_dir().ok_or(err)?;
            temp.push("mkdev");
            temp
        }
    };

    if !data_dir.is_dir() {
        fs::create_dir_all(&data_dir)?;
    }

    Ok(data_dir)
}
