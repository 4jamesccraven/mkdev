use crate::cli::Imprint;
use crate::mkdev_error::Error::{self, *};
use crate::recipe::Recipe;

use std::collections::HashMap;
use std::fs;

use ser_nix;

/// Atttempts to call recipe's imprint and save methods, returning an error message
/// on failure
pub fn imprint_recipe(args: Imprint, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    let new = Recipe::imprint(args.recipe, args.description).map_err(|why| {
        Error::from_io(
            "Unable to read current_working directory for the recipe".into(),
            &why,
        )
    })?;

    if let Some(path) = args.to_nix {
        let nix_expression = match ser_nix::to_string(&new) {
            Ok(expr) => expr,
            Err(why) => {
                return Err(SerialisationError("recipe".into(), format!("{why}")));
            }
        };

        fs::write(path, nix_expression)
            .map_err(|why| Error::from_io("unable to write to output file", &why))?;

        return Ok(());
    }

    let destructive = user_recipes.iter().any(|(recipe, _)| recipe == &new.name);

    if destructive && !args.suppress_warnings {
        return Err(DestructionWarning(new.name));
    }

    let save_location = new
        .save()
        .map_err(|error| Error::from_io("Unable to save instantiated recipe", &error))?;

    println!("{}", &save_location.display());

    Ok(())
}
