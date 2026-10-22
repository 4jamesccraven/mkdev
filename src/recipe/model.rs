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
//! Definitions for the metadata and declarative data of a standard Recipe.
use super::{Language, RecipeItem};

use std::path::Path;

use serde::{Deserialize, Serialize};

/// A mkdev recipe (v2).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Recipe {
    /// A unique identifier for the recipe.
    ///
    /// Determines both the name of the file the recipe is stored in as well as how mkdev will
    /// display information about it.
    pub name: String,
    /// A description of the recipe (Optional)
    #[serde(default = "String::new")]
    pub description: String,
    /// The programming languages (or file formats) found in the recipe's contents.
    pub languages: Vec<Language>,
    /// The contents the recipe holds.
    pub contents: Vec<RecipeItem>,
}

impl Recipe {
    /// Get the recipe's short name with no namespace component.
    pub fn shortname(&self) -> &str {
        self.name
            .rsplit_once("::")
            .map(|(_, sn)| sn)
            .unwrap_or(&self.name)
    }

    /// Get the recipe's namespace.
    ///
    /// Returns `Some(namespace)` if it belongs to a named namespace, or `None` if it belongs to the
    /// global namespace.
    pub fn namespace(&self) -> Option<&str> {
        self.name.rsplit_once("::").map(|(ns, _)| ns)
    }

    /// Replaces a specific item in a recipe with a new one.
    ///
    /// Returns `true` if the value was successfully found and replaced. Returns `false` if no matching
    /// item was found or if the item was a different type (i.e., setting a File to a Directory and vice
    /// versa)
    pub fn replace_content(&mut self, old_name: &Path, new_item: RecipeItem) -> bool {
        use crate::set_if_changed;
        match self
            .contents
            .iter_mut()
            .find(|item| &item.name() == old_name)
        {
            Some(item) => match (item, new_item) {
                (RecipeItem::File(f), RecipeItem::File(other)) => {
                    set_if_changed!(f.name, other.name) || set_if_changed!(f.content, other.content)
                }
                (RecipeItem::Directory(d), RecipeItem::Directory(other)) => {
                    set_if_changed!(*d, other)
                }
                _ => false,
            },
            None => false,
        }
    }
}
