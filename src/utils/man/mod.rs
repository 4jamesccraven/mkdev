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

//! Man page generation.
mod man5;

use crate::cli::Cli;
use crate::error::{Context, Error};
use crate::fs;

use std::boxed::Box;
use std::io::{BufWriter, ErrorKind, Write};
use std::path::{Path, PathBuf};

use clap::{Command, CommandFactory};
use clap_mangen::Man;
use clap_mangen::roff::Roff;
use rust_i18n::t;

/// Unified interface for mkdev's man pages.
pub struct MkdevMan {
    /// Underlying implementation for the selected man page.
    page: ManPage,
    /// What the man page's file name should be.
    file_name: PathBuf,
}

impl MkdevMan {
    /// Generate the man page for the top-level command, `mk`.
    pub fn command() -> Self {
        Self {
            page: Cli::command().into(),
            file_name: PathBuf::from("mk.1"),
        }
    }

    /// Generate the man page for one of mkdev's subcommands.
    ///
    /// Panics if an invalid command name is passed.
    pub fn subcommand(name: &str) -> Self {
        let command = Cli::command()
            .find_subcommand(name)
            .expect("MkdevMan::subcommand called with non-existent subcommand")
            .to_owned();

        let file_name = PathBuf::from(format!("mk-{}.1", command.get_name()));

        Self {
            page: command.into(),
            file_name,
        }
    }

    /// Generate the man page for mkdev's config file.
    pub fn config() -> Self {
        Self {
            page: man5::man5().into(),
            file_name: PathBuf::from("mkdev-config.5"),
        }
    }

    /// Write the man page as bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Writing bytes is infallible.
        _ = self.write_to(&mut buf);

        buf
    }

    /// Write the man page to its default file in the provided directory.
    pub fn write_in<P>(&self, dir: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let path = dir.as_ref().join(&self.file_name);
        let file = fs::file_create(&path, Context::Man)?;

        let mut writer = BufWriter::new(file);

        self.write_to(&mut writer)
            .and_then(|()| writer.flush())
            .map_err(|e| match e.kind() {
                ErrorKind::PermissionDenied => Error::FsDenied {
                    which: path.clone(),
                    context: Context::Man,
                },
                ErrorKind::StorageFull => {
                    let what: PathBuf = path.clone();
                    crate::die!(
                        "{}",
                        t!("warnings.storage_full", what => what.to_string_lossy())
                    )
                }
                _ => crate::borked!(e),
            })
    }

    /// Write the man page into something that implements [`Write`].
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match &self.page {
            ManPage::Clap(man) => man.render(writer),
            ManPage::Roff(roff) => roff.to_writer(writer),
        }
    }
}

/// The kinds of man page implementations used as backends.
enum ManPage {
    /// Man pages from [`clap_mangen`]
    Clap(Box<Man>),
    /// Man pages from yours truly.
    Roff(Roff),
}

impl From<Command> for ManPage {
    fn from(value: Command) -> Self {
        Self::Clap(Box::new(Man::new(value)))
    }
}

impl From<Roff> for ManPage {
    fn from(value: Roff) -> Self {
        Self::Roff(value)
    }
}
