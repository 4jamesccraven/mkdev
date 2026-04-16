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
//! std::fs functions wrapped to return mkdev Errors instead of IO Errors.
use crate::mkdev_error::{Context, Error};

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rust_i18n::t;

/// See std::fs::write.
///
/// Handles permission and storage errors only. The caller must ensure parents exist to prevent a
/// panic.
pub fn write<P, C>(path: P, contents: C, ctx: Context) -> Result<(), Error>
where
    P: AsRef<Path> + Into<PathBuf>,
    C: AsRef<[u8]>,
{
    if let Err(e) = fs::write(&path, contents) {
        match e.kind() {
            ErrorKind::PermissionDenied => {
                return Err(Error::FsDenied {
                    which: path.into(),
                    context: ctx,
                });
            }
            ErrorKind::StorageFull => {
                let what = path.into();
                crate::warning!(
                    "{}",
                    t!("warnings.storage_full", what => what.to_string_lossy())
                )
            }
            _ => crate::borked!(e),
        }
    }
    Ok(())
}

/// See std::fs::File::create.
///
/// Treats all errors as permission errors (because it is the only kind that should be able to
/// arise).
pub fn file_create<P>(path: P, ctx: Context) -> Result<fs::File, Error>
where
    P: AsRef<Path> + Into<PathBuf>,
{
    fs::File::create(&path).map_err(|_| Error::FsDenied {
        which: path.into(),
        context: ctx,
    })
}

/// See std::fs::create_dir_all.
///
/// Handles permission errors. Panics if the path is ill-formed (e.g., contains file components).
pub fn create_dir_all<P>(path: P, ctx: Context) -> Result<(), Error>
where
    P: AsRef<Path> + Into<PathBuf>,
{
    fs::create_dir_all(&path).map_err(|_| Error::FsDenied {
        which: path.into(),
        context: ctx,
    })
}

/// See std::fs::read_to_string.
///
/// Handles permission errors and UTF-8 errors only. The caller must ensure parents exist to
/// prevent a panic.
pub fn read_to_string<P>(path: P, ctx: Context) -> Result<String, Error>
where
    P: AsRef<Path> + Into<PathBuf>,
{
    fs::read_to_string(&path).map_err(|e| match e.kind() {
        ErrorKind::PermissionDenied => Error::FsDenied {
            which: path.into(),
            context: ctx,
        },
        ErrorKind::InvalidData => Error::NotUTF8 { which: path.into() },
        _ => crate::borked!(e),
    })
}

pub fn read_dir<P>(path: P, ctx: Context) -> Result<fs::ReadDir, Error>
where
    P: AsRef<Path> + Into<PathBuf>,
{
    fs::read_dir(&path).map_err(|_| Error::FsDenied {
        which: path.into(),
        context: ctx,
    })
}

/// See std::fs::remove_file.
pub fn remove_file<P>(path: P, ctx: Context) -> Result<(), Error>
where
    P: AsRef<Path> + Into<PathBuf>,
{
    fs::remove_file(&path).map_err(|e| match e.kind() {
        ErrorKind::PermissionDenied => Error::FsDenied {
            which: path.into(),
            context: ctx,
        },
        _ => crate::borked!(e),
    })
}

/// See std::env::current_dir.
///
/// Returns handles permission errors. Immediately exits the program if the cwd is not found.
pub fn current_dir() -> Result<PathBuf, Error> {
    std::env::current_dir().map_err(|e| match e.kind() {
        ErrorKind::NotFound => crate::die!("{}", t!("errors.no_cwd")),
        ErrorKind::PermissionDenied => Error::FsDenied {
            which: ".".into(),
            context: Context::Imprint,
        },
        _ => crate::borked!(e),
    })
}
