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

//! The core library for mkdev.
//!
//! This module defines the Recipe Schema and its components, as well as the most fundamental
//! operations on them.
pub mod capture;
pub mod content;
mod detection;
pub mod instantiate;
pub mod language;
mod model;
mod store;
pub mod tree;
mod version;

pub use content::RecipeItem;
pub use language::Language;
pub use model::{Recipe, RecipeId};
pub use store::RecipeStore;
