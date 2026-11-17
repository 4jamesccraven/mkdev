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

//! An iterator representing multiple successfully queried recipes.
//!
//! This `struct` is created by various 'pick' methods on the store.
use super::model::ManifestEntry;

use crate::recipe::{Recipe, RecipeId};

use std::path::Path;

/// Iterator over references to store manifest entries.
///
/// Implements convenience methods for common transformations of [`ManifestEntry`].
pub struct PickResults<'a> {
    entries: std::vec::IntoIter<&'a ManifestEntry>,
}

impl<'a> PickResults<'a> {
    /// Creates an iterator over the store paths resulting from the pick.
    pub fn paths(self) -> impl Iterator<Item = &'a Path> {
        self.map(|m| m.store_path.as_path())
    }

    /// Creates an iterator over the recipes resulting from the pick.
    pub fn recipes(self) -> impl Iterator<Item = &'a Recipe> {
        self.map(|m| &m.recipe)
    }

    /// Creates an iterator over the IDs resulting from the pick.
    pub fn ids(self) -> impl Iterator<Item = RecipeId> {
        self.map(|m| m.recipe.as_id())
    }

    /// Creates an iterator over the IDs resulting from the pick, converted to Strings.
    pub fn id_strings(self) -> impl Iterator<Item = &'a String> {
        self.recipes().map(|r| &r.name)
    }
}

impl<'a> FromIterator<&'a ManifestEntry> for PickResults<'a> {
    fn from_iter<T: IntoIterator<Item = &'a ManifestEntry>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect::<Vec<_>>().into_iter(),
        }
    }
}

impl<'a> Iterator for PickResults<'a> {
    type Item = &'a ManifestEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }
}
