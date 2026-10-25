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

//! Format for the declarative contents of a mkdev [`super::Recipe`]
//!
//! ## Comparison
//!
//! [`RecipeItem`]s are compared strictly on:
//! - Whether they are a directory or file (in that order)
//! - What the full (relative) path of the item is.
//!
//! To preserve the following relation, [`Hash`] is manually implemented.
//! File contents are explicitly discarded. This is because valid recipe data is expected to contain
//! only one item per path.
//! ```text
//! k1 == k2 -> hash(k1) == hash(k2)
//! ```
//! See [`Hash` and `Eq`](std::hash::Hash#hash-and-eq).
use std::cmp::Ordering;
use std::hash::Hash;
use std::path::{Display, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The data a mkdev recipe stores.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RecipeItem {
    File(File),
    Directory(PathBuf),
}

impl RecipeItem {
    /// Coerces to a [`Path`] slice.
    pub fn as_path(&self) -> &Path {
        match self {
            RecipeItem::File(file) => &file.name,
            RecipeItem::Directory(dir) => dir,
        }
    }

    /// Returns an object that implements [`Display`] for safely printing paths that may contain
    /// non-Unicode data.
    ///
    /// see [`Path::display`]
    pub fn display(&self) -> Display<'_> {
        self.as_path().display()
    }

    /// Construct a new [`RecipeItem::File`] variant
    pub fn file(name: PathBuf, content: String) -> Self {
        Self::File(File { name, content })
    }

    /// Constructs a new [`RecipeItem::Directory`] variant
    pub fn dir(name: PathBuf) -> Self {
        Self::Directory(name)
    }
}

/// A file.
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct File {
    pub name: PathBuf,
    pub content: String,
}

// Display
impl std::fmt::Display for RecipeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

// Comparison
impl Ord for RecipeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        fn key(r: &RecipeItem) -> (bool, &Path) {
            (matches!(r, RecipeItem::File(_)), r.as_path())
        }

        key(self).cmp(&key(other))
    }
}

impl PartialEq for RecipeItem {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for RecipeItem {}
impl PartialOrd for RecipeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Hashing
impl Hash for RecipeItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        matches!(self, RecipeItem::File(_)).hash(state);
        self.as_path().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{DefaultHasher, Hash, Hasher};

    fn hash(item: &RecipeItem) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn equal_items_have_equal_hashes() {
        let a = RecipeItem::file("test.txt".into(), "hello".into());
        let b = RecipeItem::file("test.txt".into(), "goodbye".into());

        assert_eq!(a, b);
        assert_eq!(hash(&a), hash(&b));
    }

    #[test]
    fn files_and_directories_hash_differently() {
        let file = RecipeItem::file("test.txt".into(), String::new());
        let dir = RecipeItem::dir("test.txt".into());

        assert_ne!(file, dir);
        assert_ne!(hash(&file), hash(&dir));
    }
}
