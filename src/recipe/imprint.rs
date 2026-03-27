//! Implementation of `mk imprint`.
//!
//! Imprinting is the intended way of making a new recipe. When a recipe is imprinted, mkdev walks
//! the current directory recursively and stores the relative path and contents of all text files
//! and subdirectories. Upon completion of this recursive walk, the contents are packed into a
//! recipe struct and stored to the recipe directory.
use super::{Language, Recipe, recipe_dir};
use crate::cli::Imprint;
use crate::content::{build_walk, make_contents};
use crate::ctx;
use crate::mkdev_error::{
    Error::{self, *},
    ResultExt,
};

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use hyperpolyglot::get_language_breakdown;
use ignore::Walk;

/// Imprints a recipe using arguments from the command line, and post processes it accordingly.
pub fn imprint_recipe(args: Imprint, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let walker = build_walk(&args)?;
    let new = ctx!(
        Recipe::imprint(args.recipe, args.description, walker),
        "imprinting recipe"
    )?;

    if let Some(path) = args.to_nix {
        let nix_expression = ser_nix::to_string(&new).context("recipe")?;

        ctx!(fs::write(path, nix_expression), "writing nix file")?;

        return Ok(());
    }

    let destructive = user_recipes.iter().any(|(recipe, _)| recipe == &new.name);

    if destructive && !args.suppress_warnings {
        return Err(DestructionWarning { name: new.name });
    }

    let save_location = ctx!(new.save(), "saving recipe file")?;

    println!("{}", &save_location.display());

    Ok(())
}

impl Recipe {
    /// Create a `Recipe` by imprinting/cloning the contents of the cwd
    pub fn imprint(name: String, description: Option<String>, walker: Walk) -> io::Result<Self> {
        let contents = make_contents(walker)?;

        let description = description.unwrap_or("".into());

        // Converts HashMap<&name, detected_info> -> Vec<(name, num_matching_files)>
        let mut breakdown: Vec<_> = get_language_breakdown(".")
            .iter()
            .map(|(lang, files)| (*lang, files.len()))
            .collect();

        // Sort languages by number of matching files
        breakdown.sort_by(|a, b| b.1.cmp(&a.1));

        let languages: Vec<_> = breakdown
            .iter()
            // Discard the count, as we only needed it to sort
            .map(|(lang, _)| {
                hyperpolyglot::Language::try_from(*lang)
                    .expect("detected language come pre-validated.")
            })
            .map(Language::from)
            .collect();

        Ok(Self {
            name,
            contents,
            languages,
            description,
        })
    }

    /// Save the recipe object by serialising self into the data directory
    pub fn save(&self) -> io::Result<PathBuf> {
        let mut data_dir = recipe_dir()?;

        data_dir.push(format!("{}.toml", self.name));

        fs::write(&data_dir, toml::to_string_pretty(&self).unwrap())?;

        Ok(data_dir)
    }
}
