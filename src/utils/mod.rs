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

//! A collection of miscellaneous utilities.
pub mod config;
pub mod error;
pub mod fs;
pub mod man;
pub mod replacer;

/// Assigns a value with a new one if that new value is different.
///
/// Returns `true` if the value was changed.
#[macro_export]
macro_rules! set_if_changed {
    ($l_val:expr, $r_val:expr) => {
        if $l_val != $r_val {
            $l_val = $r_val;
            true
        } else {
            false
        }
    };
}

/// Given a [`RecipeId`] and a [`RecipeStore`], determine if said recipe is externally managed
/// (i.e., is a symlink).
#[macro_export]
macro_rules! warn_if_external {
    ($id:expr, $store:ident) => {{
        use rust_i18n::t;
        if let Some(is_external) = $store.is_external($id)
            && is_external
        {
            $crate::warning!("{}", t!("recipes.external"));
        }
    }};
}

/// Formats the current file and line number.
#[macro_export]
macro_rules! ctx {
    ($msg:literal) => {
        concat!("[", file!(), ":", line!(), "] ", $msg)
    };
}
