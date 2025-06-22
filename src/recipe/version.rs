use crate::content::File;
use crate::content::RecipeItem;

use super::Language;
use super::Recipe;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(untagged)]
enum RecipeVersions {
    V2(Recipe),
    V1(RecipeV1),
}

// Version 1 //

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecipeV1 {
    pub name: String,
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

// \Version 1 //

/// Deserialises a known version of the recipe format, and converts it to the most recent version.
/// Returns None if the recipe data doesn't match any known format.
pub fn deserialise_recipe(value: &str) -> Option<Recipe> {
    toml::from_str::<RecipeVersions>(value)
        .ok()
        .map(Recipe::from)
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
