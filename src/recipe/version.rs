use super::Language;
use super::Recipe;
use crate::content::Content;

use serde::{Deserialize, Serialize};
use toml;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecipeV1 {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    pub contents: Vec<Content>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RecipeVersions {
    V2(Recipe),
    V1(RecipeV1),
}

/// Deserialises a known version of the recipe format, and converts it to the most recent version.
/// Returns None if the recipe data doesn't match any known format.
pub fn deserialise_recipe(value: &str) -> Option<Recipe> {
    toml::from_str::<RecipeVersions>(value)
        .ok()
        .map(|deserialised| Recipe::from(deserialised))
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

                Recipe {
                    name: r.name,
                    description: r.description,
                    languages,
                    contents: r.contents,
                }
            }
        }
    }
}
