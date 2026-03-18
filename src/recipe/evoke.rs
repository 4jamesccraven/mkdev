//! The implementation of `mk evoke`.
//!
//! Evoking is the "build" step for a recipe; when a recipe is selected to be evoked, its contents
//! are systematically loaded, formatted with custom substitutions, and copied into the target
//! directory.
use super::Recipe;

use crate::cli::Evoke;
use crate::config::Config;
use crate::content::RecipeItem;
use crate::mkdev_error::{
    Error::{self, *},
    ResultExt,
};
use crate::replacer::{InvalidTokenStrategy, ReplaceFmt};
use crate::warning;

use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Evokes a recipe according to arguments from the command line.
pub fn build_recipes(args: Evoke, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    // --- Error handling ---
    // There is an error if no recipes are provided
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

    // --- Replacer setup ---
    // Ensure project name is set to something
    let name = match args.name {
        Some(ref name) => name.clone(),
        None => "NAME".to_string(),
    };
    // Build to the cwd, or a directory specified by the user
    let dir = match &args.dir_name {
        Some(dir) => PathBuf::from(dir),
        None => current_dir().context("unable to get cwd")?,
    };

    let user_subs: HashMap<_, _> = Config::get()?
        .subs
        .iter()
        // Patch in reserved values
        .map(|(k, v)| match v.as_str() {
            "mk::name" => (k.clone(), format!("mk::{}", name.clone())),
            #[rustfmt::skip]
            "mk::dir" => (k.clone(), format!("mk::{}", dir.to_string_lossy())),
            _ => (k.clone(), v.clone()),
        })
        .collect();

    let re = ReplaceFmt::new(user_subs, ("{{", "}}"), InvalidTokenStrategy::Preserve);

    // --- Build ---
    let extra_args = args.clone();
    args.recipes.iter().try_for_each(|r| {
        let recipe = user_recipes
            .get(r)
            .expect("Invalid recipes should have been filtered out.");

        // Context for failure, should building fail
        let context = format!("unable to write `{}` to `{}`", recipe.name, dir.display());
        build(&dir, &recipe.contents, &extra_args, &re).context(&context)
    })
}

/// Builds a single recipe by taking in its contents and instantiating it recursively
fn build(
    dir: &Path,
    contents: &Vec<RecipeItem>,
    extra_args: &Evoke,
    re: &ReplaceFmt,
) -> io::Result<()> {
    // If the intended destination does not exist, make it.
    if !dir.is_dir() {
        fs::create_dir_all(dir)?;
    }

    for content in contents {
        let dest = dir.join(content.name());
        ensure_parent(&dest)?;

        if extra_args.verbose {
            eprintln!("{}", &dest.display());
        }

        match content {
            RecipeItem::File(file) => {
                // perform substitutions on the name and contents
                let name = re.replace_with(&dest.to_string_lossy(), run_shell);
                let content = re.replace_with(&file.content, run_shell);

                // Stop if a file would be overwritten unless the user has explicitly suppressed
                // it.
                if dest.is_file() && !extra_args.suppress_warnings {
                    use std::io::ErrorKind::*;
                    return Err(io::Error::new(
                        AlreadyExists,
                        format!("'{}' already exists.", file.name.display()),
                    ));
                }

                fs::write(&name, content)?;
            }
            RecipeItem::Directory(dir_name) => {
                // Perform substitutions on the dirname
                let name = re.replace_with(&dir_name.to_string_lossy(), run_shell);
                let dest = dir.join(name);

                fs::create_dir_all(&dest)?;
            }
        }
    }

    Ok(())
}

/// Ensures that all parent directories of a file exist.
fn ensure_parent(path: &Path) -> io::Result<()> {
    let parent = match path.parent() {
        Some(p) => p,
        None => return Ok(()),
    };

    if !parent.is_dir() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

/// Runs the provided command.
///
/// Calculated reserved values (prefixed with 'mk::') are immediately dumped instead.
fn run_shell(cmd: &str) -> Option<String> {
    // Handle reserved names.
    if cmd.starts_with("mk::") {
        let out = cmd.strip_prefix("mk::").unwrap().to_string();
        return Some(out);
    }

    let output = Command::new("sh").arg("-c").arg(cmd).output().ok();

    match output {
        Some(output) => {
            // Convert to utf-8 text and strip the trailing newline (if there is one).
            let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            if stdout.ends_with('\n') {
                stdout.pop();
            }
            Some(stdout)
        }
        None => {
            warning!("could not run command '{}'", cmd);
            None
        }
    }
}
