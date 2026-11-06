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

//! Functions for querying the store and resolving [`RecipeId`]s.
use super::model::{ManifestEntry, RecipeStore};
use super::pick_results::PickResults;

use crate::error::{Error, Subject};
use crate::recipe::RecipeId;

impl RecipeStore {
    // ------------------------------------------------------------
    // ID Resolution and Querying
    // ------------------------------------------------------------

    /// Resolves a potentially incomplete [`RecipeId`] against the manifest, returning a reference
    /// to the canonical key in the manifest.
    ///
    /// If no [`ManifestEntry`] matches the key, an [`Error::Invalid`] is returned. If more than one
    /// is a match, [`Error::AmbiguousShortRecipe`] is returned.
    ///
    /// ## Resolution Order
    /// The `recipe_id` argument does not have to be an exact match---the recipe store tries several
    /// strategies to match the key it receives, choosing the first to succeed:
    /// 1. An exact match always succeeds.
    /// 2. The input is interpreted as a namespace, and the default is returned if it exists (e.g.,
    ///    "foo" is interpreted as "foo::default").
    /// 3. Any recipe that's shortname matches is marked as a candidate (candidate examples for
    ///    input "foo": "python::foo", "my_stuff::foo", etc.).
    ///
    /// If the input contains both a namespace _and_ a recipe name, and the exact match fails, the
    /// other strategies are not employed, and an error is immediately returned.
    pub(super) fn pick_raw(&self, recipe_id: &RecipeId) -> Result<&RecipeId, Error> {
        // Always prefer an exact match.
        if let Some((key, _)) = self.manifest.get_key_value(recipe_id) {
            return Ok(key);
        }

        // If the exact match failed with both parts specified, that recipe does not exist.
        if recipe_id.namespace.is_some() || recipe_id.explicit_global {
            return Err(Error::Invalid {
                subject: Subject::Recipe,
                examples: Some(vec![recipe_id.to_string()]),
            });
        }

        // See if the remainder part was intended as namespace:
        let remainder_as_namespace = RecipeId {
            namespace: Some(recipe_id.shortname.clone()),
            shortname: "default".into(),
            explicit_global: false,
        };
        if let Some((key, _)) = self.manifest.get_key_value(&remainder_as_namespace) {
            return Ok(key);
        }

        // As a last resort, see if the remainder alone uniquely identifies a recipe.
        let potential_matches: Vec<_> = self
            .manifest
            .keys()
            .filter(|k| k.shortname == recipe_id.shortname)
            .collect();

        match potential_matches.len() {
            0 => Err(Error::Invalid {
                subject: Subject::Recipe,
                examples: Some(vec![recipe_id.to_string()]),
            }),
            1 => {
                let entry = potential_matches.into_iter().next().unwrap();
                Ok(entry)
            }
            2.. => Err(Error::AmbiguousShortRecipe {
                query: recipe_id.to_string(),
                possibilities: potential_matches
                    .into_iter()
                    .map(|k| self.manifest[k].recipe.name.clone())
                    .collect(),
            }),
        }
    }

    /// Query the store for a single manifest entry.
    pub fn pick_one<K>(&self, recipe_id: K) -> Result<&ManifestEntry, Error>
    where
        K: Into<RecipeId>,
    {
        let key = recipe_id.into();
        self.pick_raw(&key).map(|id| &self.manifest[id])
    }

    /// Query the store for a reference to the canonical key matching the input key.
    pub fn pick_id<K>(&self, recipe_id: K) -> Result<&RecipeId, Error>
    where
        K: Into<RecipeId>,
    {
        let recipe_id = recipe_id.into();
        self.pick_raw(&recipe_id)
    }

    /// Query the store for several manifest entries.
    pub fn pick_many<I, K>(&self, recipe_ids: I) -> Result<PickResults<'_>, Error>
    where
        I: IntoIterator<Item = K>,
        K: Into<RecipeId>,
    {
        Ok(recipe_ids
            .into_iter()
            .map(|id| self.pick_raw(&id.into()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|id| &self.manifest[id])
            .collect())
    }

    /// Query the store for all manifest entries.
    pub fn pick_all(&self) -> PickResults<'_> {
        self.manifest.values().collect()
    }

    /// Query the store for all manifest entries in the namespace.
    pub fn pick_namespace(&self, namespace: &str) -> PickResults<'_> {
        self.manifest
            .iter()
            .filter_map(|(k, v)| {
                let ns = k.namespace.as_ref()?;
                (ns == namespace).then_some(v)
            })
            .collect()
    }

    /// Query the store to see if an ID exists.
    pub fn recipe_exists<K>(&self, recipe_id: K) -> bool
    where
        K: Into<RecipeId>,
    {
        let key = recipe_id.into();
        self.pick_raw(&key).is_ok()
    }

    /// Query the store to see if a namespace exists.
    ///
    /// Returns `true` if at least one recipe in the store has the specified namespace.
    pub fn namespace_exists(&self, namespace: &str) -> bool {
        self.manifest
            .values()
            .any(|m| m.recipe.namespace() == Some(namespace))
    }
}
