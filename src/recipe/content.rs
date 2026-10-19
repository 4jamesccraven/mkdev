//! ...

use std::cmp::Ordering;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The data a mkdev recipe stores.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RecipeItem {
    File(File),
    Directory(PathBuf),
}

impl RecipeItem {
    /// Returns the name of the `RecipeItem`.
    pub fn name(&self) -> String {
        let name = match self {
            RecipeItem::File(file) => file.name.to_string_lossy(),
            RecipeItem::Directory(dir) => dir.to_string_lossy(),
        };

        name.into()
    }

    /// Constructs a new [`RecipeItem::Directory`] variant
    pub fn dir(name: PathBuf) -> Self {
        Self::Directory(name)
    }

    /// Construct a new [`RecipeItem::File`] variant
    pub fn file(name: PathBuf, content: String) -> Self {
        Self::File(File { name, content })
    }
}

/// A file.
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct File {
    pub name: PathBuf,
    pub content: String,
}

impl std::fmt::Display for RecipeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
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

impl Ord for RecipeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        fn key(r: &RecipeItem) -> (bool, String) {
            (matches!(r, RecipeItem::File(_)), r.name())
        }

        key(self).cmp(&key(other))
    }
}

impl std::hash::Hash for RecipeItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RecipeItem::File(f) => {
                0u8.hash(state);
                f.name.hash(state);
            }
            RecipeItem::Directory(d) => {
                1u8.hash(state);
                d.hash(state);
            }
        }
    }
}
