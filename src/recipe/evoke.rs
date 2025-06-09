use super::Recipe;

use crate::cli::Evoke;
use crate::content::Content;
use crate::mkdev_error::{
    Error::{self, *},
    ResultExt,
};
use crate::subs::Replacer;

use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::io;
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
        None => current_dir().context("unable to get cwd")?,
    };

    let extra_args = args.clone();
    args.recipes.iter().try_for_each(|r| {
        let recipe = user_recipes
            .get(r)
            .expect("Invalid recipes should have been filtered out.");

        // Context for failure, should building fail
        let context = format!("Unable to write `{}` to `{}`", recipe.name, dir.display());
        build(&dir, &recipe.contents, &extra_args, &re).context(&context)
    })?;

    Ok(())
}

/// Builds a single recipe by taking in its contents and instantiating it recursively
fn build(
    dir: &PathBuf,
    contents: &Vec<Content>,
    extra_args: &Evoke,
    re: &Replacer,
) -> io::Result<()> {
    if !dir.is_dir() {
        fs::create_dir_all(&dir)?;
    }

    for content in contents {
        let mut path = dir.clone();
        let project_name = extra_args
            .name
            .as_ref()
            .expect("Name is converted to a Some variant in the `build_recipes` wrapper function.");
        let name = re.sub(&content.get_name(), &project_name, dir);
        path.push(name);

        if path.is_file() && !extra_args.suppress_warnings {
            use std::io::ErrorKind::*;
            return Err(io::Error::new(
                AlreadyExists,
                format!("'{}' already exists.", path.display()),
            ));
        }

        if extra_args.verbose {
            println!("{}", path.display());
        }

        match content {
            Content::File(file) => {
                let content = re.sub(&file.content, &project_name, dir);
                fs::write(&path, content)?;
            }
            Content::Directory(directory) => {
                fs::create_dir_all(&path)?;
                build(&dir, &directory.files, &extra_args, re)?;
            }
        }
    }

    Ok(())
}
