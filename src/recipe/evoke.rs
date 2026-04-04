// mkdev - Save your boilerplate instead of writing it
// Copyright (C) 2026  James C. Craven <4jamesccraven@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//! The implementation of `mk evoke`.
//!
//! Evoking is the "build" step for a recipe; when a recipe is selected to be evoked, its contents
//! are systematically loaded, formatted with custom substitutions, and copied into the target
//! directory.
use super::Recipe;

use crate::cli::Evoke;
use crate::config::Config;
use crate::content::{File, RecipeItem};
use crate::fs_wrappers;
use crate::mkdev_error::{
    Error::{self, *},
    Subject,
};
use crate::recipe::{OnConflict, instantiate_contents};
use crate::replacer::{InvalidTokenStrategy, ReplaceFmt};
use crate::warning;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use rust_i18n::t;

/// Evokes a recipe according to arguments from the command line.
pub fn build_recipes(args: Evoke, user_recipes: HashMap<String, Recipe>) -> Result<(), Error> {
    // Make sure that the recipes past are valid.
    validate_args(&args, &user_recipes)?;

    // Build to the cwd, or a directory specified by the user
    let dir = match &args.dir_name {
        Some(dir) => PathBuf::from(dir),
        None => fs_wrappers::current_dir()?,
    };

    let re = evocation_resolver(&args, &dir)?;

    args.recipes.iter().try_for_each(|r| {
        let recipe = user_recipes.get(r).expect("recipes were validated above.");
        let contents = resolve_items(&recipe.contents, &re);
        let on_conflict = if args.suppress_warnings {
            OnConflict::Overwrite
        } else {
            OnConflict::Guard
        };

        // Context for failure, should building fail
        instantiate_contents(&dir, &contents, on_conflict, args.verbose).inspect_err(|_| {
            warning!(
                "{}",
                t!("errors.evoke", recipe => recipe.name, target => dir.display())
            )
        })
    })
}

/// Applies a replacer to all the names and contents of a collection of RecipeItems, returning a
/// new owned collection of them.
fn resolve_items(contents: &[RecipeItem], re: &ReplaceFmt) -> Vec<RecipeItem> {
    contents
        .iter()
        .map(|item| match item {
            RecipeItem::File(file) => RecipeItem::File(File {
                name: re.replace_path_with(run_shell, &file.name),
                content: re.replace_with(run_shell, &file.content),
            }),
            RecipeItem::Directory(dir) => {
                RecipeItem::Directory(re.replace_path_with(run_shell, dir))
            }
        })
        .collect()
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
            warning!("{}", t!("warnings.child_failed", child => cmd));
            None
        }
    }
}

/// Verifies that at least one valid recipes was passed at the command line.
fn validate_args(args: &Evoke, user_recipes: &HashMap<String, Recipe>) -> Result<(), Error> {
    // There is an error if no recipes are provided
    if args.recipes.is_empty() {
        return Err(NoneSpecified {
            subject: Subject::Recipes,
        });
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
        let subject = match non_existant_recipes.len() {
            1 => Subject::Recipe,
            2.. => Subject::Recipes,
            _ => unreachable!(),
        };
        return Err(Invalid {
            subject,
            examples: Some(non_existant_recipes),
        });
    }

    Ok(())
}

/// Sets up the replacefmt used during evocation.
fn evocation_resolver(args: &Evoke, dir: &Path) -> Result<ReplaceFmt, Error> {
    // Ensure project name is set to something
    let name = match args.name {
        Some(ref name) => name.clone(),
        None => "NAME".to_string(),
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

    Ok(ReplaceFmt::new(
        user_subs,
        ("{{", "}}"),
        InvalidTokenStrategy::Preserve,
    ))
}
