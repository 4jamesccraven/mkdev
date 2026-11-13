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

//! The mkdev store.
//!
//! The store is a central repository that manages [`Recipe`] discovery and modification. The store
//! controls saving, deletion, and editing of recipes during the lifetime of the program, and
//! provides methods for querying its data.
//!
//! ## Example Usage
//! ```no_run
//! use mkdev::recipe::RecipeStore;
//!
//! // Standard initialisation.
//! let store = RecipeStore::gather().unwrap();
//!
//! // Get all recipes.
//! let all_recipes: Vec<_> = store.pick_all().recipes().collect();
//!
//! // Get one.
//! let my_recipe_foo = store.pick_one("foo").unwrap().recipe.clone();
//! ```
mod backup;
mod inspect;
mod model;
mod mutate;
mod pick;
mod pick_results;

#[expect(unused, reason = "docs only")]
use crate::recipe::Recipe;

pub use model::RecipeStore;
