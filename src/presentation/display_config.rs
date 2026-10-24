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
//! The configuration for displaying a single recipe.
use confique::Config as Confique;
use serde::{Deserialize, Serialize};

#[derive(Confique, Serialize, Deserialize, Clone, Debug)]
pub struct DisplayConfig {
    // --- Top Level ---
    /// Text that joins (comes between) formatted recipes.
    ///
    /// Default:
    /// "\n"
    #[serde(default = "default_recipes_join")]
    pub recipes_join: String,
    /// Extra text that should come after all recipes are printed.
    ///
    /// Default:
    /// "\n"
    #[serde(default = "default_recipes_suffix")]
    pub recipes_suffix: String,
    /// Whether the CLI flag `--no-description` is set by default.
    ///
    /// Default:
    /// None (determined by CLI only)
    pub show_descriptions: Option<bool>,
    /// How an individual recipe should be formatted. The variables available are "{name}", the
    /// formatted name of the recipe; "{langs}", the formatted list of languages present in the
    /// recipe; and "{desc}" the formatted description of the recipe.
    ///
    /// Default:
    /// "{name} ({langs}){desc}"
    #[serde(default = "default_recipe_fmt")]
    pub recipe_fmt: String,

    // --- Namespace ---
    /// How to format the namespace divider. Use "{namespace}" to reference the namespace. If the
    /// namespace_fmt is empty, namespace dividers are not shown. Leading newlines are stripped for
    /// the first given namespace.
    ///
    /// Default:
    /// "\n--- {namespace} ---"
    #[serde(default = "default_namespace_fmt")]
    pub namespace_fmt: String,
    /// Whether the namespace divider should always show even if only the global namespace exists.
    ///
    /// Default:
    /// false
    #[serde(default = "default_namespace_show_always")]
    pub namespace_show_always: bool,

    // --- Name ---
    /// How to format the name of the recipe. Use "{name}" to reference the full recipe name (with
    /// the namespace), "{namespace}" to reference the namespace, or "{shortname}" to reference the
    /// short name.
    ///
    /// Default:
    /// "{shortname}"
    #[serde(default = "default_name_fmt")]
    pub name_fmt: String,
    /// Whether the name should be bolded.
    ///
    /// Default:
    /// true
    #[serde(default = "default_name_bold")]
    pub name_bold: bool,

    // --- Description ---
    /// How to format the description of the recipe. (use "{name}" to reference the raw
    /// description).
    ///
    /// Default:
    /// "\n  {desc}"
    #[serde(default = "default_desc_fmt")]
    pub desc_fmt: String,

    // --- Languages ---
    /// How each individual language should be formatted. (use "{lang}" to reference the raw
    /// language name).
    ///
    /// Default:
    /// "{lang}"
    #[serde(default = "default_lang_fmt")]
    pub lang_fmt: String,
    /// Whether a language should be colourised.
    ///
    /// Default:
    /// true
    #[serde(default = "default_lang_colour")]
    pub lang_colour: bool,
    /// Text that joins formatted languages.
    ///
    /// Default:
    /// " "
    #[serde(default = "default_langs_join")]
    pub langs_join: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            recipes_join: default_recipes_join(),
            recipes_suffix: default_recipes_suffix(),
            show_descriptions: None,
            recipe_fmt: default_recipe_fmt(),
            namespace_fmt: default_namespace_fmt(),
            namespace_show_always: default_namespace_show_always(),
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
    pub fn default_recipes_join()          -> String { "\n".to_string()                     }
    pub fn default_recipes_suffix()        -> String { "\n".to_string()                     }
    pub fn default_recipe_fmt()            -> String { "{name} ({langs}){desc}".to_string() }
    pub fn default_namespace_fmt()         -> String { "\n--- {namespace} ---".to_string()  }
    pub fn default_namespace_show_always() -> bool   { false                                }
    pub fn default_name_fmt()              -> String { "{shortname}".to_string()            }
    pub fn default_name_bold()             -> bool   { true                                 }
    pub fn default_desc_fmt()              -> String { "\n  {desc}".to_string()             }
    pub fn default_lang_fmt()              -> String { "{lang}".to_string()                 }
    pub fn default_lang_colour()           -> bool   { true                                 }
    pub fn default_langs_join()            -> String { " ".to_string()                      }
}
