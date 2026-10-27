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

//! The error type for mkdev.
use std::path::PathBuf;

use rust_i18n::t;

#[derive(Clone, Debug)]
pub enum Error {
    /// Recipe(s) were not specified.
    NoneSpecified { subject: Subject },

    /// The value is invalid in the context it was received.
    Invalid {
        subject: Subject,
        examples: Option<Vec<String>>,
    },

    /// The query does not uniquely identify a single recipe.
    AmbiguousShortRecipe {
        query: String,
        possibilities: Vec<String>,
    },

    /// The action would be destructive.
    DestructiveAction { name: String },

    /// The user-defined exclude arguments were malformed.
    ///
    /// Note: this is effectively an ignore error, which is itself effectively an IO error.
    Exclude { cause: String },

    /// The filesystem operation lacked the necessary privileges to complete.
    FsDenied { which: PathBuf, context: Context },

    /// The file's contents were not UTF-8 encoded.
    NotUTF8 { which: PathBuf },

    /// The input device was not a TTY (during interactive mode).
    NotTTY,

    /// The user canceled an interactive process.
    InteractiveCancelled,

    /// The value failed to serialise.
    #[allow(unused)]
    Serialisation {
        which: PathBuf,
        cause: String,
        context: Context,
    },

    /// The value failed to deserialise.
    Deserialisation {
        which: PathBuf,
        cause: String,
        context: Context,
    },
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoneSpecified { subject } => {
                write!(f, "{}", t!("errors.none_specified", subject => subject))
            }
            Error::Invalid { subject, examples } => {
                let base = t!("errors.invalid", subject => subject);
                match examples.as_deref() {
                    Some(eg) => {
                        write!(
                            f,
                            "{base}:{}{}",
                            if eg.len() > 1 { "\n" } else { " " },
                            eg.join("\n")
                        )
                    }
                    None => {
                        write!(f, "{base}")
                    }
                }
            }
            Error::AmbiguousShortRecipe {
                query,
                possibilities,
            } => {
                let ps = possibilities.join(", ");
                write!(f, "{} '{query}' ({ps})", t!("errors.ambiguous"))
            }
            Error::Exclude { cause } => {
                write!(f, "{}: {cause}", t!("errors.exclude"))
            }
            Error::FsDenied { which, context } => write!(
                f,
                "{}",
                t!("errors.fs_denied", which => which.to_string_lossy(), context => context),
            ),
            Error::NotUTF8 { which } => write!(
                f,
                "{}",
                t!("errors.not_utf8", file => which.to_string_lossy())
            ),
            Error::NotTTY => write!(f, "{}", t!("errors.not_tty")),
            Error::InteractiveCancelled => write!(f, "{}", t!("errors.interrupted")),
            Error::DestructiveAction { name } => {
                write!(f, "{}", t!("errors.destruction", name => name))
            }
            Error::Serialisation {
                which,
                cause,
                context,
            } => {
                write!(
                    f,
                    "{}\n{cause}",
                    t!("errors.serialise", which => which.to_string_lossy(), context => context)
                )
            }
            Error::Deserialisation {
                which,
                cause,
                context,
            } => {
                write!(
                    f,
                    "{}\n{cause}",
                    t!("errors.deserialise", which => which.to_string_lossy(), context => context)
                )
            }
        }
    }
}

/// A context for more generic error types.
#[derive(Clone, Copy, Debug)]
pub enum Context {
    Config,
    Delete,
    Evoke,
    Gather,
    Imprint,
    Man,
    Tempfile,
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Context::Config => t!("contexts.config"),
                Context::Delete => t!("contexts.delete"),
                Context::Evoke => t!("contexts.evocation"),
                Context::Gather => t!("context.gather"),
                Context::Imprint => t!("contexts.imprint"),
                Context::Man => t!("contexts.man"),
                Context::Tempfile => t!("contexts.tempfile"),
            }
        )
    }
}

/// A subject for the Invalid and NoneSpecified error types.
#[derive(Clone, Copy, Debug)]
pub enum Subject {
    Recipe,
    Recipes,
}

impl Subject {
    /// Gets the correct plurality from a count.
    ///
    /// Panics if 0 is passed.
    pub fn from_count(count: usize) -> Self {
        match count {
            1 => Self::Recipe,
            2.. => Self::Recipes,
            0 => crate::borked!(count),
        }
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Recipe => t!("subject.recipe"),
                Self::Recipes => t!("subject.recipes"),
            }
        )
    }
}

/// Print a warning to the stderr.
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{} {}", "[ warning ]".yellow(), format_args!($($arg)*));
    }};
}

/// Print an error message and exit the program early.
#[macro_export]
macro_rules! die {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{} {}", "[  error  ]".red(), format_args!($($arg)*));
        std::process::exit(1);
    }};
}

/// For use when an unresolvable, unexpected error occurs.
#[macro_export]
macro_rules! borked {
    ($err:expr) => {{
        $crate::die!("{} unexpected error: please make an issue at https://github.com/4jamesccraven/mkdev !!\n{}", $crate::ctx!(""), $err);
    }};
}

/// Formats the current file and line number.
#[macro_export]
macro_rules! ctx {
    ($msg:literal) => {
        concat!("[", file!(), ":", line!(), "] ", $msg)
    };
}

impl From<ignore::Error> for Error {
    fn from(e: ignore::Error) -> Self {
        Error::Exclude {
            cause: e.to_string(),
        }
    }
}

impl From<inquire::error::InquireError> for Error {
    fn from(value: inquire::error::InquireError) -> Self {
        match value {
            inquire::InquireError::NotTTY => Error::NotTTY,
            inquire::InquireError::OperationCanceled => Error::InteractiveCancelled,
            inquire::InquireError::OperationInterrupted => Error::InteractiveCancelled,
            _ => borked!(value),
        }
    }
}
