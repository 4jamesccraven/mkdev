//! Implementats a unified error type, a unified logging interface, and conversions from common
//! error types.
use rust_i18n::t;
use std::io;

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

    /// Wraps `std::io::Error`.
    Io {
        context: &'static str,
        cause: String,
    },

    /// Indicates that a value failed to serialise.
    Serialisation { what: &'static str, cause: String },

    /// Indicates that a value failed to deserialise.
    Deserialisation { what: &'static str, cause: String },
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
            Error::DestructionWarning { name } => {
                write!(f, "{}", t!("errors.destruction", name => name))
            }
            Error::Io { context, cause } => {
                write!(f, "{context}: {cause}")
            }
            Error::Serialisation { what, cause } => {
                write!(
                    f,
                    "{}",
                    t!("errors.serialise", what => what, cause => cause)
                )
            }
            Error::Deserialisation { what, cause } => {
                write!(
                    f,
                    "{}",
                    t!("errors.deserialise", what => what, cause => cause)
                )
            }
        }
    }
}

/// A subject for the Invalid and NoneSpecified error types.
#[derive(Clone, Debug)]
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
        eprintln!("{}: {}", "[mkdev warning]".yellow(), format_args!($($arg)*));
    }};
}

/// Print an error message and exit the program early.
#[macro_export]
macro_rules! die {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{}: {}", "[mkdev error]".red(), format_args!($($arg)*));
        std::process::exit(1);
    }};
}

/// A helper for providing debug info (file/line number) either on a raw error message or on a
/// result that implements `ResultExt`.
#[macro_export]
macro_rules! ctx {
    ($msg:literal) => {
        concat!("[", file!(), ":", line!(), "] ", $msg)
    };
    ($res:expr, $msg:literal) => {{
        use $crate::mkdev_error::ResultExt;
        ResultExt::context($res, ctx!($msg))
    }};
}

/// Convert the error type of a result to `mkdev_error::Error`
pub trait ResultExt<T> {
    /// Converts the `Result` using a context message.
    fn context(self, context: &'static str) -> Result<T, Error>;
}

impl<T> ResultExt<T> for Result<T, io::Error> {
    fn context(self, context: &'static str) -> Result<T, Error> {
        self.map_err(|e| Error::Io {
            context,
            cause: e.to_string(),
        })
    }
}

impl<T> ResultExt<T> for Result<T, ser_nix::Error> {
    fn context(self, context: &'static str) -> Result<T, Error> {
        self.map_err(|e| Error::Serialisation {
            what: context,
            cause: e.to_string(),
        })
    }
}

impl<T> ResultExt<T> for Result<T, toml::de::Error> {
    fn context(self, context: &'static str) -> Result<T, Error> {
        self.map_err(|e| Error::Deserialisation {
            what: context,
            cause: e.message().to_string(),
        })
    }
}

impl From<ignore::Error> for Error {
    fn from(e: ignore::Error) -> Self {
        let context = "exclude flags";
        Error::Io {
            context,
            cause: e.to_string(),
        }
    }
}
