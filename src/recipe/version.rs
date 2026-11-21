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

//! Backwards compatibility layer for older recipe versions.
use super::{Language, Recipe, RecipeItem, content::File};

use crate::error::{Context, Error};

#[expect(unused, reason = "docs only")]
use crate::store::RecipeStore;

use serde::{Deserialize, Serialize};

/// The maximum version of the original recipe format.
pub const RECIPE_VERSION_MAX: u8 = 2;

/// Represents the backing format of a recipe, accessible via [`RecipeStore`] entries.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum RecipeKind {
    /// A declarative recipe that is created by deserialising something with serde.
    DeclarativeSerde {
        /// The version the recipe was deserialised from.
        ///
        /// Versions:
        /// 1. Original, recursive storage of contents.
        /// 2. Modern, flat storage of contents.
        version: u8,
    },
}

impl std::fmt::Display for RecipeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeclarativeSerde { version: v } => write!(f, "Declarative (serde v{v})"),
        }
    }
}

impl Default for RecipeKind {
    fn default() -> Self {
        Self::DeclarativeSerde {
            version: RECIPE_VERSION_MAX,
        }
    }
}

/// Deserialises a known version of the recipe format, and converts it to the most recent version.
///
/// Returns [`Err`] if the recipe data doesn't match any known format.
pub fn deserialise_declarative_recipe(value: &str) -> Result<(Recipe, RecipeKind), Error> {
    toml::from_str::<RecipeVersions>(value)
        .map(RecipeVersions::into_modern)
        .map_err(|e| Error::Deserialisation {
            cause: e.to_string(),
            context: Context::Gather,
        })
}
/// Untagged helper enum to transparently deserialise [`RecipeKind::SerdeDeclarative`] versions.
#[derive(Deserialize)]
#[serde(untagged)]
enum RecipeVersions {
    V2(Recipe),
    V1(RecipeV1),
}

impl RecipeVersions {
    /// Converts `self` into the most modern format, preserving versioning info.
    pub fn into_modern(self) -> (Recipe, RecipeKind) {
        let kind = RecipeKind::DeclarativeSerde {
            version: self.as_version_no(),
        };

        (self.into(), kind)
    }

    /// Coerces `self` to a version number.
    pub fn as_version_no(&self) -> u8 {
        match self {
            Self::V2(_) => 2,
            Self::V1(_) => 1,
        }
    }
}

impl From<RecipeVersions> for Recipe {
    fn from(value: RecipeVersions) -> Self {
        use RecipeVersions::*;
        match value {
            V2(r) => r,
            V1(r) => {
                // V1 has hardcoded string languages, so those need to be converted to a language
                // struct if possible
                let languages = r
                    .languages
                    .into_iter()
                    .map(|string| Language::from(string.as_str()))
                    .collect();

                // V1's content was recursive and needs to be flattened.
                let contents = flatten_v1_recursive(r.contents);

                Recipe {
                    name: r.name,
                    description: r.description,
                    languages,
                    contents,
                }
            }
        }
    }
}

// ------------------------------------------------------------
//  V1 Format
// ------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecipeV1 {
    pub name: String,
    #[serde(default = "String::new")]
    pub description: String,
    pub languages: Vec<String>,
    pub contents: Vec<ContentV1>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ContentV1 {
    File(FileV1),
    Directory(DirectoryV1),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileV1 {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectoryV1 {
    pub name: String,
    pub files: Vec<ContentV1>,
}

/// Unravels nested V1 content into a flat Vec.
fn flatten_v1_recursive(old: Vec<ContentV1>) -> Vec<RecipeItem> {
    let mut out = vec![];

    for item in old {
        match item {
            ContentV1::File(f) => {
                let new_f = File {
                    name: f.name.into(),
                    content: f.content,
                };

                out.push(RecipeItem::File(new_f));
            }
            ContentV1::Directory(d) => {
                let mut extended = vec![RecipeItem::Directory(d.name.into())];
                extended.extend(flatten_v1_recursive(d.files));
                out.extend(extended);
            }
        }
    }

    out.sort();
    out
}
