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
//! An internal function that allows shell completions to detect the user's recipes.
use crate::recipe::Recipe;

use clap_complete::engine::CompletionCandidate;

/// An argument completer for the CLI that returns matching recipe names.
pub fn recipe_completer(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let mut completions = vec![];

    if let Some(current) = current.to_str()
        && let Ok(recipes) = Recipe::gather()
    {
        recipes
            .keys()
            .filter(|&c| c.starts_with(current))
            .for_each(|c: &String| {
                completions.push(CompletionCandidate::new(c));
            });
    }

    completions
}
