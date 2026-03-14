//! Implementats a unified error type, a unified logging interface, and conversions from common
//! error types.
use std::io;

/// mkdev's error type.
#[derive(thiserror::Error, Clone, Debug)]
pub enum Error {
    /// Indicates that something wasn't specified when it should be.
    #[error("no {0} specified.")]
    NoneSpecified(String),

    /// Indicates that a value is invalid in the context it was passed.
    #[error("invalid {0}{examples}", examples = {
        match .1 {
            Some(eg) => format!(":\n{}", eg.join("\n")),
            None => String::from("."),
        }
    })]
    Invalid(String, Option<Vec<String>>),

    /// Wraps `std::io::Error`.
    #[error("{0}: {1}")]
    Io(String, String),

    /// Indicates that a value failed to serialise.
    #[error("failed to serialise {0}: {1}")]
    Serialisation(String, String),

    /// Indicates that a value failed to deserialise.
    #[allow(unused)]
    #[error("failed to deserialise {0}: {1}")]
    Deserialisation(String, String),

    /// Indicates that an action would be destructive.
    #[error("'{0}' already exists. Use -s to overwrite.")]
    DestructionWarning(String),
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

/// Convert the error type of a result to `mkdev_error::Error`
pub trait ResultExt<T> {
    /// Converts the `Result` using a context message.
    fn context(self, s: &str) -> Result<T, Error>;
}

impl<T> ResultExt<T> for Result<T, io::Error> {
    fn context(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::Io(s.to_string(), e.to_string()))
    }
}

impl<T> ResultExt<T> for Result<T, ser_nix::Error> {
    fn context(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::Serialisation(s.to_string(), e.to_string()))
    }
}

impl<T> ResultExt<T> for Result<T, toml::de::Error> {
    fn context(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::Deserialisation(s.to_string(), e.message().to_string()))
    }
}

impl From<ignore::Error> for Error {
    fn from(e: ignore::Error) -> Self {
        Error::Io("error with exclude flags".into(), e.to_string())
    }
}
