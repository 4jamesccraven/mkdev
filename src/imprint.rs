use crate::cli::Imprint;
use crate::mkdev_error::{
    Error::{self, *},
    ResultExt,
};
use crate::recipe::Recipe;

use std::collections::HashMap;
use std::fs;

use ser_nix;

/// Atttempts to call recipe's imprint and save methods, returning an error message
/// on failure
pub fn imprint_recipe(args: Imprint, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let new = Recipe::imprint(args.recipe, args.description)
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
