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

//! Implentation of `mk examine`.
//!
//! For examining the exact contents of a recipe on disk.
use super::MkdevCommand;

use crate::error::Error;
use crate::recipe::RecipeStore;

use std::io::Write;
use std::process::{Command, Stdio};

use clap::{Parser, ValueHint};

#[derive(Parser, Debug)]
pub struct Examine {
    /// The recipe(s) whose contents should be displayed.
    pub recipes: Vec<String>,

    /// Show the path to the recipe instead of its contents.
    #[arg(short, long)]
    pub paths: bool,

    /// Optional commmand that receives the recipe contents from STDIN.
    #[arg(last = true, value_hint = ValueHint::CommandWithArguments, conflicts_with = "paths")]
    pub pipe_into: Vec<String>,
}

impl MkdevCommand for Examine {
    fn execute(self, store: RecipeStore) -> Result<(), Error> {
        exmaine_command(self, store)
    }
}

pub fn exmaine_command(args: Examine, store: RecipeStore) -> Result<(), Error> {
    let pick_results = store.pick_many(&args.recipes)?;

    // Show the paths only and exit if `--paths` is passed.
    // If there are no recipes specified, all are used for
    if args.paths {
        let pick_results = if args.recipes.is_empty() {
            store.pick_all()
        } else {
            pick_results
        };

        pick_results
            .paths()
            .for_each(|p| println!("{}", p.display()));

        return Ok(());
    }

    if let Some((prog, pipe_args)) = args.pipe_into.split_first() {
        // Spawn child and pipe into it...
        let mut rx = Command::new(prog)
            .args(pipe_args)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| Error::SpawnFailure {
                what: prog.clone(),
                why: e,
            })?;

        let mut rx_write = rx.stdin.take().expect("stdin provided above.");

        let write_status = pick_results.ids().try_for_each(|id| {
            let data = store.recipe_data_bytes(id)?;
            rx_write
                .write_all(&data)
                .map_err(|e| Error::PipeFailure { why: e })
        });

        drop(rx_write);
        let wait_status = rx.wait().map_err(|e| Error::WaitFailure { why: e })?;
        let exit_status = write_status.map(|_| wait_status)?;

        if !exit_status.success() {
            return Err(Error::UnsuccessfulChild {
                code: exit_status.code(),
            });
        }

        Ok(())
    } else {
        // ... or send it to stdout directly.
        pick_results.ids().try_for_each(|id| store.recipe_cat(id))
    }
}
