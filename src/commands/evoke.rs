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
use crate::cli::Evoke;
use crate::config::Config;
use crate::error::Error;
use crate::recipe::content::{File, RecipeItem};
use crate::recipe::instantiate::{InstantiateOpts, OnConflict, instantiate_contents};
use crate::recipe::{Recipe, RecipeStore};
use crate::replacer::{InvalidTokenStrategy, ReplaceFmt};
use crate::{menus, warning};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use rust_i18n::t;

pub fn evoke_recipe(args: Evoke, store: RecipeStore) -> Result<(), Error> {
    // Determine what goes in the `mk::name` substitution.
    let project_name = match args.name {
        Some(ref name) => name.clone(),
        None => "NAME".to_string(),
    };

    // Get user selection via the command line, or from an interactive menu.
    let query = if !args.interactive {
        &args.recipes
    } else {
        let choices = store.pick_all().id_strings().collect();
        &menus::select_recipes(choices)?
    };

    // Validate the user selection and configure instantiation.
    let targets: Vec<_> = store.pick_many(query)?.recipes().cloned().collect();
    let opts = InstantiateOpts::from(args);

    // Resolve and instantiate the targets.
    let resolver = init_resolver(&project_name, &opts.root_dir)?;
    let recipes = resolve_targets(&resolver, targets.as_slice());
    recipes.into_iter().try_for_each(|recipe| {
        instantiate_contents(&recipe.contents, &opts).inspect_err(|_| {
            warning!(
                "{}",
                t!("errors.evoke", recipe => recipe.name, target => opts.root_dir.as_path().display())
            )
        })
    })
}

/// Resolves substitutions in the names and contents of the selected recipes.
fn resolve_targets(resolver: &ReplaceFmt, targets: &[Recipe]) -> Vec<Recipe> {
    targets
        .iter()
        .map(|recipe| Recipe {
            contents: resolve_items(resolver, &recipe.contents),
            ..recipe.clone()
        })
        .collect()
}

/// Resolves substitutions in a recipe's files and directories.
fn resolve_items(re: &ReplaceFmt, contents: &[RecipeItem]) -> Vec<RecipeItem> {
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
/// If `cmd` is prefixed with "mk::", the prefix is stripped and the value is returned as-is,
/// without shelling out.
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

/// Initialises the evocation resolver.
fn init_resolver<P>(name: &str, dir: P) -> Result<ReplaceFmt, Error>
where
    P: AsRef<Path>,
{
    // Make `dir` absolute before proceeding, if possible.
    let user_subs: HashMap<_, _> = Config::get()?
        .subs
        .iter()
        // Patch in reserved values
        .map(|(k, v)| match v.as_str() {
            "mk::name" => (k.clone(), format!("mk::{}", name)),
            "mk::dir" => (k.clone(), format!("mk::{}", dir.as_ref().display())),
            _ => (k.clone(), v.clone()),
        })
        .collect();

    Ok(ReplaceFmt::new(
        user_subs,
        ("{{", "}}"),
        InvalidTokenStrategy::Preserve,
    ))
}

impl From<Evoke> for InstantiateOpts {
    fn from(args: Evoke) -> Self {
        Self {
            on_conflict: OnConflict::allow_overwrite(args.suppress_warnings),
            verbose: args.verbose,
            root_dir: args
                .dir_name
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        }
    }
}
