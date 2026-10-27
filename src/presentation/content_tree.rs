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

//! Tree-like output for the contents of a mkdev recipe.
use crate::recipe::{Recipe, RecipeItem};

use std::collections::BTreeMap;
use std::path::Path;

use colored::Colorize;

pub fn recipe_treefmt(recipe: &Recipe) -> String {
    let mut out = format!("{}\n", recipe.name.bold().blue());
    let contents = format_content_tree(&recipe.contents);
    out.push_str(&contents);

    out
}

/// Create a String that represents the file system akin to the output of the program "tree"
fn format_content_tree(files: &[RecipeItem]) -> String {
    let tree = build_content_tree(files);
    format_tree(&tree, "".into())
}

const GAP: &str = "    ";
const WIRE: &str = "│   ";
const CONNECTOR: &str = "├── ";
const CAP: &str = "└── ";

/// Displays a tree structure.
fn format_tree<T>(contents: &[T], prefix: String) -> String
where
    T: AsRef<TreeContent>,
{
    let mut contents: Vec<&TreeContent> = contents.iter().map(AsRef::as_ref).collect();
    contents.sort_by_key(|&c| (c.is_file, &c.name));

    let mut rec_iter = contents.iter().peekable();
    let mut out = String::new();

    while let Some(content) = rec_iter.next() {
        // Like `ls`, we use blue to differentiate directories from files.
        let name = if content.is_file {
            content.name.normal()
        } else {
            content.name.blue()
        };

        // If this content is the last of its group, the line reflects that by not appearing to
        // continue downwards.
        let is_last = rec_iter.peek().is_none();
        let line = if is_last { CAP } else { CONNECTOR };

        // Format the current recipe item's name into the buffer.
        let new_line = format!(
            "{}{}{}\n",
            prefix.truecolor(128, 128, 128),
            line.truecolor(128, 128, 128),
            name
        );
        out.push_str(&new_line);

        // If this item has children, recursively add those as well.
        if !content.children.is_empty() {
            // If this is the last item, its children's prefix is just indentation. Otherwise, a
            // line visually represents that there are more entries after this recursion.
            let new_prefix = format!("{}{}", prefix, if is_last { GAP } else { WIRE });

            // Recurse.
            let rec = format_tree(content.values().as_slice(), new_prefix);
            out.push_str(&rec);
        }
    }
    out
}

/// Nests the default, flat recipe content structure into a recursive tree.
fn build_content_tree(files: &[RecipeItem]) -> Vec<TreeContent> {
    use RecipeItem::*;
    // Create an intermediate tree
    let mut root = TreeContent::default();

    // Insert each file into the tree
    for file in files {
        match file {
            File(file) => {
                root.insert(file.name.as_path(), true);
            }
            Directory(name) => {
                root.insert(name, false);
            }
        }
    }

    root.children.into_values().collect()
}

/// Intermediate representation of the file tree
#[derive(Debug, Clone, Default)]
struct TreeContent {
    /// What should be printed to represent this [`RecipeItem`].
    name: String,
    /// Whether the content is a file.
    is_file: bool,
    /// Children of this directory, if it is a directory.
    children: BTreeMap<String, TreeContent>,
}

impl TreeContent {
    /// Create a new [`TreeContent`] node.
    fn new(name: String) -> Self {
        TreeContent {
            is_file: false,
            name,
            children: BTreeMap::new(),
        }
    }

    /// Insert a path into the tree by breaking it into its components.
    fn insert(&mut self, path: &Path, is_file: bool) {
        let mut current = self;

        // Break the path into its components, and insert them one-by-one.
        for comp in path.components() {
            let name = comp.as_os_str().to_string_lossy().into_owned();
            current = current
                .children
                .entry(name.clone())
                .or_insert_with(|| TreeContent::new(name));
        }

        // Note whether the leaf node is a file.
        if is_file {
            current.is_file = true;
        }
    }

    /// Get a vector of references to this node's children.
    fn values(&self) -> Vec<&TreeContent> {
        self.children.values().collect()
    }
}

impl AsRef<TreeContent> for TreeContent {
    fn as_ref(&self) -> &TreeContent {
        self
    }
}
