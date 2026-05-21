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
use crate::mkdev_error::{
    Error::{self, *},
    Subject,
};
use crate::recipe::{OnConflict, instantiate_contents};
use crate::replacer::{InvalidTokenStrategy, ReplaceFmt};
use crate::warning;
use crate::{fs_wrappers, menus};

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use rust_i18n::t;

/// The set of parameters that define how evocation should be carried out.
pub struct EvocationCtx {
    /// Command line arguments.
    args: Evoke,
    /// Where fully resolved evocation results are placed.
    target_dir: PathBuf,
    /// All the user's recipes.
    recipes: HashMap<String, Recipe>,
    /// The names of the recipes that were selected.
    target_recipes: Vec<String>,
    /// Recipes that have had their contents fully resolved.
    resolved_recipes: Vec<Recipe>,
    /// The name of the fully resolved and instantiated recipe, as provided by the CLI argument
    /// `--name`.
    name: String,
}

impl EvocationCtx {
    /// Generate the context for evocation from a collection of command line arguments.
    pub fn from_args(args: Evoke, user_recipes: HashMap<String, Recipe>) -> Result<Self, Error> {
        if args.interactive {
            return Self::from_prompt(args, user_recipes);
        }

        // There is an error if no recipes are provided
        if args.recipes.is_empty() {
            return Err(NoneSpecified {
                subject: Subject::Recipes,
            });
        }

        // Validate & store the name of the requested recipes.
        let target_recipes = Recipe::pick_many(&user_recipes, &args.recipes)
            .map(|rs| rs.into_iter().map(|r| r.name.clone()).collect::<Vec<_>>())?;

        // User specified target or default to CWD.
        let target_dir = match &args.dir_name {
            Some(dir) => PathBuf::from(dir),
            None => fs_wrappers::current_dir()?,
        };

        let mut ctx = Self {
            name: Self::unwrap_name(&args),
            args,
            target_dir,
            recipes: user_recipes,
            resolved_recipes: Vec::with_capacity(target_recipes.len()),
            target_recipes,
        };
        ctx.resolve_targets()?;

        Ok(ctx)
    }

    /// Create the context for evocation partially interactively.
    pub fn from_prompt(args: Evoke, user_recipes: HashMap<String, Recipe>) -> Result<Self, Error> {
        let target_recipes = menus::evoke(&user_recipes)?;
        let target_dir = fs_wrappers::current_dir()?;

        let mut ctx = Self {
            name: Self::unwrap_name(&args),
            args,
            target_dir,
            recipes: user_recipes,
            resolved_recipes: Vec::with_capacity(target_recipes.len()),
            target_recipes,
        };
        ctx.resolve_targets()?;

        Ok(ctx)
    }

    /// Perform evocation as defined by the context.
    pub fn evoke(&mut self) -> Result<(), Error> {
        let on_conflict = match self.args.suppress_warnings {
            true => OnConflict::Overwrite,
            false => OnConflict::Guard,
        };

        self.resolved_recipes.iter().try_for_each(|recipe| {
            instantiate_contents(
                &self.target_dir,
                &recipe.contents,
                on_conflict,
                self.args.verbose,
            )
            .inspect_err(|_| {
                warning!(
                    "{}",
                    t!("errors.evoke", recipe => recipe.name, target => self.target_dir.display())
                )
            })
        })
    }

    /// Resolve the contents of all the recipes and store them.
    fn resolve_targets(&mut self) -> Result<(), Error> {
        let re = self.init_resolver()?;
        self.resolved_recipes.clear();

        for recipe_name in &self.target_recipes {
            let recipe = self
                .recipes
                .get(recipe_name)
                .expect("These are checked during initialisation.");

            self.resolved_recipes.push(Recipe {
                contents: resolve_items(&recipe.contents, &re),
                ..recipe.clone()
            })
        }

        Ok(())
    }

    /// Initialises the evocation resolver.
    ///
    /// The resolver is the mechanism by which templated text is replaced with actual values.
    fn init_resolver(&self) -> Result<ReplaceFmt, Error> {
        let user_subs: HashMap<_, _> = Config::get()?
            .subs
            .iter()
            // Patch in reserved values
            .map(|(k, v)| match v.as_str() {
                "mk::name" => (k.clone(), format!("mk::{}", self.name.clone())),
                "mk::dir" => (
                    k.clone(),
                    format!("mk::{}", self.target_dir.to_string_lossy()),
                ),
                _ => (k.clone(), v.clone()),
            })
            .collect();

        Ok(ReplaceFmt::new(
            user_subs,
            ("{{", "}}"),
            InvalidTokenStrategy::Preserve,
        ))
    }

    /// Helper method to ensure that users not specifying `--name` is resolved consistently across
    /// contexts.
    fn unwrap_name(args: &Evoke) -> String {
        match &args.name {
            Some(n) => n.clone(),
            None => String::from("NAME"),
        }
    }
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
