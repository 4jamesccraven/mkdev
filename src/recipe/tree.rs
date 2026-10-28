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

use crate::ctx;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Recursive representation of a recipe's contents.
#[derive(Debug, Clone, Default)]
pub struct RecipeItemTree {
    /// What should be printed to represent this [`RecipeItem`].
    pub name: String,
    /// The full path from the original item.
    pub path: PathBuf,
    /// File contents, if it is a file.
    pub contents: Option<String>,
    /// Children of this directory, if it is a directory.
    pub children: BTreeMap<String, RecipeItemTree>,
}

impl RecipeItemTree {
    /// Create a new [`RecipeItemTree`] node.
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let file_name = path
            .as_ref()
            .file_name()
            .expect(ctx!("Tree received invalid node `..` (parent dir)"));

        RecipeItemTree {
            name: format!("{}", file_name.display()),
            path: path.as_ref().to_path_buf(),
            contents: None,
            children: BTreeMap::new(),
        }
    }

    /// Returns [`true`] if this node is a file.
    pub fn is_file(&self) -> bool {
        self.contents.is_some()
    }

    /// Returns [`true`] if this node is a directory.
    pub fn is_dir(&self) -> bool {
        self.contents.is_none()
    }

    /// Insert a path into the tree by breaking it into its components.
    pub fn insert(&mut self, path: &Path, file_contents: Option<String>) {
        let mut current_node = self;
        let mut current_path = PathBuf::new();

        // Break the path into its components, and insert them one-by-one.
        for comp in path.components() {
            let name = format!("{}", comp.as_os_str().display());
            current_path.push(&comp);

            current_node = current_node
                .children
                .entry(name.clone())
                .or_insert_with(|| RecipeItemTree::new(current_path.clone()));
        }

        // Add the optional file contents to determine if this is a file.
        current_node.contents = file_contents;
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
                    root.insert(file.name.as_path(), Some(file.content.clone()));
                }
                Directory(name) => {
                    root.insert(name, None);
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
