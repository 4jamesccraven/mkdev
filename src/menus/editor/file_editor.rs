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
//
//
// Portions of this file are derived from the `inquire` crate:
// https://github.com/mikaelmello/inquire
//
// Original work licensed under the MIT License.
// Copyright (c) 2021 Mikael Mello
//
// See inquire.MIT.txt for the full license text.

//! Support for opening the editor.
//!
//! Why not use the one the one from `inquire`? It strips newlines from the user's "input," meaning
//! that genuinely desired newlines do not make it into the final recipe, and thus silently changes
//! user data.
use std::io::Write;
use std::path::Path;
use std::{env, process};

use inquire::error::InquireResult;
use tempfile::NamedTempFile;

/// Edits long-form text inside the user's preferred $EDITOR.
pub struct FileEditor<'a> {
    buf: String,
    ext: &'a str,
}

impl<'a> FileEditor<'a> {
    pub fn new(current: &str, ext: &'a str) -> Self {
        Self {
            buf: current.to_string(),
            ext,
        }
    }

    /// Run the file editor.
    pub fn run(mut self) -> InquireResult<String> {
        let tmp = self.init_tmp()?;
        self.spawn_editor(tmp.path())?;
        self.buf = std::fs::read_to_string(tmp.path())?;

        Ok(self.buf)
    }

    /// Initialise a temporary file to edit into.
    fn init_tmp(&self) -> InquireResult<NamedTempFile> {
        let mut tmp_file = tempfile::Builder::new()
            .prefix("tmp-")
            .suffix(&format!(".{}", self.ext))
            .rand_bytes(10)
            .tempfile()?;

        tmp_file.write_all(self.buf.as_bytes())?;
        tmp_file.flush()?;

        Ok(tmp_file)
    }

    /// Run the user's editor on a path.
    fn spawn_editor(&mut self, file: &Path) -> InquireResult<()> {
        process::Command::new(Self::get_editor())
            .arg(file)
            .spawn()?
            .wait()?;

        Ok(())
    }

    /// Get the user's preferred editor or default to nano.
    fn get_editor() -> String {
        let mut default_editor = String::from("nano");

        if let Ok(editor) = env::var("EDITOR")
            && !editor.is_empty()
        {
            default_editor = editor;
        }

        if let Ok(editor) = env::var("VISUAL")
            && !editor.is_empty()
        {
            default_editor = editor;
        }

        default_editor
    }
}
