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
use std::io;
use std::path::PathBuf;

use rust_i18n::t;

#[derive(Debug)]
pub enum Error {
    // ------------------------------------------------------------
    //  Recipe Resolution
    // ------------------------------------------------------------
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

    // ------------------------------------------------------------
    //  CLI Mode
    // ------------------------------------------------------------
    /// The action would be destructive.
    DestructiveAction { name: String },

    /// The user-defined exclude arguments were malformed.
    ///
    /// Note: this is effectively an ignore error, which is itself effectively an IO error.
    Exclude { cause: String },

    // ------------------------------------------------------------
    //  Interactive Mode
    // ------------------------------------------------------------
    /// The input device was not a TTY (during interactive mode).
    NotTTY,

    /// The user canceled an interactive process.
    InteractiveCancelled,

    // ------------------------------------------------------------
    //  (De)serialisation
    // ------------------------------------------------------------
    /// The value failed to serialise.
    Serialisation { cause: String, context: Context },

    /// The value failed to deserialise.
    Deserialisation { cause: String, context: Context },

    // ------------------------------------------------------------
    //  Child Processes
    // ------------------------------------------------------------
    /// Could not spawn the child process.
    SpawnFailure { what: String, why: io::Error },

    /// Could not write to child process.
    PipeFailure { why: io::Error },

    /// Could not collect child exit status.
    WaitFailure { why: io::Error },

    /// Child exited with a non-zero status.
    UnsuccessfulChild { code: Option<i32> },

    // ------------------------------------------------------------
    //  Filesystem
    // ------------------------------------------------------------
    /// The filesystem operation lacked the necessary privileges to complete.
    FsDenied { which: PathBuf, context: Context },

    /// The file's contents were not UTF-8 encoded.
    NotUTF8 { which: PathBuf },
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Recipe Resolution
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
            // CLI Mode
            Error::DestructiveAction { name } => {
                write!(f, "{}", t!("errors.destruction", name => name))
            }
            Error::Exclude { cause } => {
                write!(f, "{}: {cause}", t!("errors.exclude"))
            }
            // Interactive Mode
            Error::NotTTY => write!(f, "{}", t!("errors.not_tty")),
            Error::InteractiveCancelled => write!(f, "{}", t!("errors.interrupted")),
            // Serde
            Error::Serialisation { cause, context } => {
                write!(f, "{}\n{cause}", t!("errors.serialise", context => context))
            }
            Error::Deserialisation { cause, context } => {
                write!(
                    f,
                    "{}\n{cause}",
                    t!("errors.deserialise", context => context)
                )
            }
            // Children
            Error::SpawnFailure { what, why } => {
                write!(f, "{}:\n{}", t!("errors.spawn_failure", what => what), why)
            }
            Error::PipeFailure { why } => write!(f, "{}:\n{}", t!("errors.pipe_failure"), why),
            Error::WaitFailure { why } => write!(f, "{}:\n{}", t!("errors.wait_failure"), why),
            Error::UnsuccessfulChild { code } => match code {
                Some(code) => write!(f, "{}: {}", t!("errors.child_failed_with"), code),
                None => write!(f, "{}", t!("errors.child_failed_unknown")),
            },
            // FS
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
        }
    }
}

/// A context for more generic error types.
#[derive(Clone, Copy, Debug)]
pub enum Context {
    Backup,
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
                Context::Backup => t!("contexts.backup"),
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
    Index,
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
                Self::Index => t!("subject.index"),
            }
        )
    }
}

/// Print a warning to stderr.
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
