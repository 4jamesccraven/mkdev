//! Implementats a unified error type, a unified logging interface, and conversions from common
//! error types.
use std::path::PathBuf;

use rust_i18n::t;

/// mkdev's error type.
#[derive(Clone, Debug)]
pub enum Error {
    /// Indicates that something wasn't specified when it should be.
    NoneSpecified { subject: Subject },

    /// Indicates that a value is invalid in the context it was passed.
    Invalid {
        subject: Subject,
        examples: Option<Vec<String>>,
    },

    /// Indicates that an action would be destructive.
    DestructionWarning { name: String },

    /// An error arising from building user-provided exclusions during imprinting.
    ///
    /// Note: this is effectively an ignore error, which is itself effectively an IO error.
    Exclude { cause: String },

    /// Tried to create a directory without valid permissions to do so.
    FsDenied { which: PathBuf, context: Context },

    /// An error arising from trying to read a non-UTF-8 file.
    NotUTF8 { which: PathBuf },

    /// Indicates that a value failed to serialise.
    #[allow(unused)]
    Serialisation {
        which: PathBuf,
        cause: String,
        context: Context,
    },

    /// Indicates that a value failed to deserialise.
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
            Error::DestructionWarning { name } => {
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

/// A helper for providing debug info (file/line number) either on a raw error message or on a
/// result that implements `ResultExt`.
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
