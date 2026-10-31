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

use crate::cli::{Cli, SubcommandType};
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

    /// Generate the man page for one of mkdev's subcommands, using the [`SubcommandType`] enum.
    pub fn subcommand_discriminant(value: SubcommandType) -> Self {
        let value_str = value.to_string().to_lowercase();
        Self::subcommand_name(&value_str)
    }

    /// Generate the man page for one of mkdev's subcommands.
    ///
    /// Panics if an invalid command name is passed.
    pub fn subcommand_name(name: &str) -> Self {
        let command = Cli::command()
            .find_subcommand(name)
            .expect("MkdevMan::subcommand called with non-existent subcommand")
            .to_owned();

        let file_name = format!("mk-{}", command.get_name());
        let command = command.display_name(&file_name);
        let file_name = PathBuf::from(file_name).with_extension("1");

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

    // Generate an iterator over all possible subcommands
    //
    // The full list consists of:
    // - The top-level command `mk`
    // - The config file
    // - Every subcommand
    pub fn all() -> impl Iterator<Item = Self> {
        let sc_iter = std::iter::once(Cli::command()).flat_map(|cmd| {
            cmd.get_subcommands()
                .map(|sc| MkdevMan::subcommand_name(sc.get_name()))
                .collect::<Vec<_>>()
        });

        [MkdevMan::command(), MkdevMan::config()]
            .into_iter()
            .chain(sc_iter)
    }

    /// Display the man page to the user.
    pub fn open(&self) -> Result<(), Error> {
        let bytes = self.to_bytes();

        let mut file = tempfile::NamedTempFile::new().map_err(|_| Error::FsDenied {
            which: "/tmp".into(),
            context: Context::Tempfile,
        })?;

        file.write_all(&bytes).expect("idek");

        std::process::Command::new("man")
            .arg("-l")
            .arg(file.path())
            .status()
            .expect("Could not run man!");

        Ok(())
    }

    /// Write the man page as bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Writing bytes is infallible.
        _ = self.write_to(&mut buf);

        buf
    }

    /// Writes all man pages to the provided directory.
    pub fn write_all(dir: impl AsRef<Path>) -> Result<(), Error> {
        // Ensure the path exists.
        let path = dir.as_ref();
        fs::create_dir_all(path, Context::Man)?;

        Self::all().try_for_each(|manpage| manpage.write_in(path))
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
