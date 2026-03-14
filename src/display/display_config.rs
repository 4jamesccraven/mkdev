//! The configuration for displaying a single recipe.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisplayConfig {
    // --- Top Level ---
    /// Text that joins formatted recipes
    /// Default: "\n"
    #[serde(default = "default_recipes_join")]
    pub recipes_join: String,
    /// Whether the CLI flag `--no-description` is set by default.
    /// Default: None (determined by CLI only)
    pub show_descriptions: Option<bool>,
    /// How an individual recipe should be formatted.
    /// Default: "{name} ({langs}){desc}"
    #[serde(default = "default_recipe_fmt")]
    pub recipe_fmt: String,

    // --- Name ---
    /// How to format the name of the recipe.
    /// Default: "{name}"
    #[serde(default = "default_name_fmt")]
    pub name_fmt: String,
    /// Whether the name should be bolded.
    /// Default: true
    #[serde(default = "default_name_bold")]
    pub name_bold: bool,

    // --- Description ---
    /// How to format the description of the recipe.
    /// Default: "\n  {desc}"
    #[serde(default = "default_desc_fmt")]
    pub desc_fmt: String,

    // --- Languages ---
    /// How each individual language should be formatted.
    /// Default: "{lang}"
    #[serde(default = "default_lang_fmt")]
    pub lang_fmt: String,
    /// Whether a language should be colourised.
    /// Default: true
    pub lang_colour: bool,
    /// Text that joins formatted languages.
    /// Default: " "
    #[serde(default = "default_langs_join")]
    pub langs_join: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            recipes_join: default_recipes_join(),
            show_descriptions: None,
            recipe_fmt: default_recipe_fmt(),
            name_fmt: default_name_fmt(),
            name_bold: default_name_bold(),
            desc_fmt: default_desc_fmt(),
            lang_fmt: default_lang_fmt(),
            lang_colour: default_lang_colour(),
            langs_join: default_langs_join(),
        }
    }
}

use config_defaults::*;
#[rustfmt::skip]
mod config_defaults {
    //! Source of truth for `DisplayConfig::default` implementation
    pub fn default_recipes_join() -> String { "\n".to_string()                     }
    pub fn default_recipe_fmt()   -> String { "{name} ({langs}){desc}".to_string() }
    pub fn default_name_fmt()     -> String { "{name}".to_string()                 }
    pub fn default_name_bold()    -> bool   { true                                 }
    pub fn default_desc_fmt()     -> String { "\n  {desc}".to_string()             }
    pub fn default_lang_fmt()     -> String { "{lang}".to_string()                 }
    pub fn default_lang_colour()  -> bool   { true                                 }
    pub fn default_langs_join()   -> String { " ".to_string()                      }
}
