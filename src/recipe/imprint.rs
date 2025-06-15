use super::{recipe_dir, Language, Recipe};
use crate::cli::Imprint;
use crate::content::{build_walk, make_contents};
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
use ser_nix;

/// Atttempts to call recipe's imprint and save methods, returning an error message
/// on failure
pub fn imprint_recipe(args: Imprint, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let walker = build_walk(args.exclude)?;
    let new = Recipe::imprint(args.recipe, args.description, walker)
        .context("Unable to read current_working directory for the recipe")?;

    if let Some(path) = args.to_nix {
        let nix_expression = ser_nix::to_string(&new).context("recipe")?;

        fs::write(path, nix_expression).context("unable to write to output file")?;

        return Ok(());
    }

    let destructive = user_recipes.iter().any(|(recipe, _)| recipe == &new.name);

    if destructive && !args.suppress_warnings {
        return Err(DestructionWarning(new.name));
    }

    let save_location = new.save().context("Unable to save instantiated recipe")?;

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
                    .expect("Languages from `get_language_breakdown` are guaranteed")
            })
            .map(|lang| Language::from(lang))
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
