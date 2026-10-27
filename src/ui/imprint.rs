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

//! An interactive workflow for the [`Imprint`] command.
use super::prompts::{get_recipe_description, get_recipe_name, select_contents};

// Docs-only
#[expect(unused)]
use crate::commands::Imprint;

use crate::config::Config;
use crate::error::Error;
use crate::fs::current_dir;
use crate::recipe::Recipe;
use crate::recipe::capture::capture_contents;

/// Interactively imprint a recipe from the current directory.
pub fn run_imprinter() -> Result<Recipe, Error> {
    // Ensure the config is loaded in memory before proceeding.
    let _config = Config::get()?;
    let mut recipe = Recipe::default();
    let cwd = current_dir()?;

    let name = get_recipe_name()?;
    recipe.name = name;

    let description = get_recipe_description()?;
    recipe.description = description;

    let default_contents = capture_contents(&cwd)?;
    recipe.contents = select_contents(default_contents)?;

    Ok(recipe)
}
