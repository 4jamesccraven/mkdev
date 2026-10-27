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

//! High level operations on the store.
//!
//! These operations are expected to be consumed as CLI subcommands, but they are also conceptual.
//! They bridge the gap between what the user specifies and low-level implementation details.
mod alter;
mod delete;
mod evoke;
mod imprint;
mod list;

pub use alter::*;
pub use delete::*;
pub use evoke::*;
pub use imprint::*;
pub use list::*;

use crate::{error::Error, recipe::RecipeStore};

/// A trait that represents a command to be run against a [`RecipeStore`].
///
/// This enforces the type signature of the command, ensuring uniformity.
pub trait StoreCommand {
    fn execute(self, store: RecipeStore) -> Result<(), Error>;
}
