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
use crate::recipe::{Recipe, tree::RecipeItemTree};

use colored::Colorize;

/// Display a Recipe and its contents in a tree-like format.
pub fn recipe_treefmt(recipe: &Recipe) -> String {
    let mut out = format!("{}\n", recipe.name.bold().blue());
    let roots: Vec<_> = RecipeItemTree::from_iter(recipe.contents.as_slice()).into_roots();

    out.push_str(format_tree(&roots, "".into()).as_str());
    out
}

const GAP: &str = "    ";
const WIRE: &str = "│   ";
const CONNECTOR: &str = "├── ";
const CAP: &str = "└── ";

/// Formats a content tree.
fn format_tree<T>(contents: &[T], prefix: String) -> String
where
    T: AsRef<RecipeItemTree>,
{
    let mut contents: Vec<&RecipeItemTree> = contents.iter().map(AsRef::as_ref).collect();
    contents.sort_by_key(|&c| (c.is_file(), &c.name));

    let mut rec_iter = contents.iter().peekable();
    let mut out = String::new();

    while let Some(content) = rec_iter.next() {
        // Like `ls`, we use blue to differentiate directories from files.
        let name = if content.is_file() {
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
        if content.is_dir() && !content.children.is_empty() {
            // If this is the last item, its children's prefix is just indentation. Otherwise, a
            // line visually represents that there are more entries after this recursion.
            let new_prefix = format!("{}{}", prefix, if is_last { GAP } else { WIRE });

            // Recurse.
            let rec = format_tree(content.as_roots().as_slice(), new_prefix);
            out.push_str(&rec);
        }
    }
    out
}
