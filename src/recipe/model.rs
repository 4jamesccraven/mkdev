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
//!
//! # Recipe IDs
//! mkdev recipes are always part of exactly one space: either the global, nameless one, or a custom
//! one specified by the user, e.g.:
//! - `[global namespace]::foo`
//! - `python::foo`
//!
//! The namespace in an id is everything, including colon characters, preceeding the **last** double
//! colon. Below is a typical example of a valid ID.
//! ```text
//! foo_namespace::bar_recipe
//! ^            ^ ^
//! |            | |
//! |            | +-- recipe name
//! |            +---- delimiter
//! +----------------- namespace
//! ```
//!
//! If the delimiter appears multiple times, only the last one is respected:
//! ```text
//! let the id equal "foo::bar::baz::qux":
//!
//! foo::bar::baz ::qux
//! ^^^^^^^^^^^^^ ^ ^
//! |             | |
//! |             | +-- recipe name: qux
//! |             +---- the only delimiter that is honoured
//! +------------------ namespace: "foo::bar::baz"
//! ```
use super::{Language, RecipeItem};

use std::{fmt::Display, path::Path};

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
    /// Get `self.name` as structured data.
    pub fn as_id(&self) -> RecipeId {
        self.into()
    }

    /// Get the recipe's short name with no namespace component.
    #[expect(unused)]
    pub fn shortname(&self) -> &str {
        destructure_id_str(&self.name).1
    }

    /// Get the recipe's namespace.
    ///
    /// Returns `Some(namespace)` if it belongs to a named namespace, or `None` if it belongs to the
    /// global namespace.
    pub fn namespace(&self) -> Option<&str> {
        destructure_id_str(&self.name).0
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
            .find(|item| item.as_path() == old_name)
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

// RecipeId

/// A destructured unique identifier for a recipe.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecipeId {
    pub namespace: Option<String>,
    pub shortname: String,
}

impl RecipeId {
    /// Destructure the ID into its constituent parts.
    pub fn into_parts(self) -> (Option<String>, String) {
        (self.namespace, self.shortname)
    }

    /// Destrucutre the ID into its constituent parts, converting the [`Option`] namespace to an
    /// empty [`String`] on [`None`].
    pub fn into_parts_infallibe(self) -> (String, String) {
        let (maybe_ns, shortname) = self.into_parts();
        (maybe_ns.unwrap_or_else(String::new), shortname)
    }
}

impl Display for RecipeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.namespace {
            Some(ns) => write!(f, "{}::{}", ns, self.shortname),
            None => write!(f, "{}", self.shortname),
        }
    }
}

impl<T: AsRef<str>> From<T> for RecipeId {
    fn from(id: T) -> Self {
        let (namespace, shortname) = destructure_id_str(id.as_ref());

        Self {
            namespace: namespace.map(str::to_owned),
            shortname: shortname.to_owned(),
        }
    }
}

impl From<&Recipe> for RecipeId {
    fn from(recipe: &Recipe) -> Self {
        Self::from(recipe.name.as_str())
    }
}

// Helper Functions

/// Destructures a recipe id string.
fn destructure_id_str(id: &str) -> (Option<&str>, &str) {
    match id.rsplit_once("::") {
        Some((namespace, shortname)) => (Some(namespace), shortname),
        None => (None, id),
    }
}
