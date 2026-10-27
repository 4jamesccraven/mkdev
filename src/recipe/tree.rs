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

//! Tree-like representation for [`RecipeItem`]s.
use super::RecipeItem;

use std::collections::BTreeMap;
use std::path::Path;

/// Recursive representation of a recipe's contents.
#[derive(Debug, Clone, Default)]
pub struct RecipeItemTree {
    /// What should be printed to represent this [`RecipeItem`].
    pub name: String,
    /// Whether the content is a file.
    pub is_file: bool,
    /// Children of this directory, if it is a directory.
    pub children: BTreeMap<String, RecipeItemTree>,
}

impl RecipeItemTree {
    /// Create a new [`RecipeItemTree`] node.
    pub fn new(name: String) -> Self {
        RecipeItemTree {
            is_file: false,
            name,
            children: BTreeMap::new(),
        }
    }

    /// Insert a path into the tree by breaking it into its components.
    pub fn insert(&mut self, path: &Path, is_file: bool) {
        let mut current = self;

        // Break the path into its components, and insert them one-by-one.
        for comp in path.components() {
            let name = comp.as_os_str().to_string_lossy().into_owned();
            current = current
                .children
                .entry(name.clone())
                .or_insert_with(|| RecipeItemTree::new(name));
        }

        // Note whether the leaf node is a file.
        if is_file {
            current.is_file = true;
        }
    }

    /// Returns a vector of the tree's roots, consuming the tree.
    pub fn into_roots(self) -> Vec<RecipeItemTree> {
        self.into_iter().collect()
    }

    /// Returns a vector of references to the tree's roots.
    pub fn as_roots(&self) -> Vec<&RecipeItemTree> {
        self.iter().collect()
    }

    /// Returns an iterator over the tree's children.
    pub fn iter(&self) -> impl Iterator<Item = &RecipeItemTree> {
        self.children.values()
    }
}

impl IntoIterator for RecipeItemTree {
    type Item = RecipeItemTree;
    type IntoIter = std::collections::btree_map::IntoValues<String, RecipeItemTree>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_values()
    }
}

impl<'a> FromIterator<&'a RecipeItem> for RecipeItemTree {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a RecipeItem>,
    {
        use RecipeItem::*;
        // Create an intermediate tree
        let mut root = RecipeItemTree::default();

        // Insert each file into the tree
        for item in iter {
            match item {
                File(file) => {
                    root.insert(file.name.as_path(), true);
                }
                Directory(name) => {
                    root.insert(name, false);
                }
            }
        }

        root
    }
}

impl AsRef<RecipeItemTree> for RecipeItemTree {
    fn as_ref(&self) -> &RecipeItemTree {
        self
    }
}
