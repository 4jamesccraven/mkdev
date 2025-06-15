use std::io;

use ignore;
use ser_nix;
use thiserror;

#[derive(thiserror::Error, Clone, Debug)]
pub enum Error {
    #[error("No {0} specified.")]
    NoneSpecified(String),

    #[error("invalid {0}{examples}", examples = {
        match .1 {
            Some(eg) => format!(":\n{}", eg.join("\n")),
            None => String::from("."),
        }
    })]
    Invalid(String, Option<Vec<String>>),

    #[error("{0}: {1}")]
    Io(String, String),

    #[error("failed to serialise {0}: {1}")]
    SerialisationError(String, String),

    #[allow(unused)]
    #[error("failed to deserialise {0}: {1}")]
    DeserialisationError(String, String),

    #[error("'{0}' already exists. Use -s to overwrite.")]
    DestructionWarning(String),
}

/// Print a warning to the user
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{}: {}", "[mkdev warning]".yellow(), format_args!($($arg)*));
    }};
}

/// Exit the program early
#[macro_export]
macro_rules! die {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{}: {}", "[mkdev error]".red(), format_args!($($arg)*));
        std::process::exit(1);
    }};
}

pub trait ResultExt<T> {
    fn context(self, s: &str) -> Result<T, Error>;
}

impl<T> ResultExt<T> for Result<T, io::Error> {
    fn context(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::Io(s.to_string(), e.to_string()))
    }
}

impl<T> ResultExt<T> for Result<T, ser_nix::Error> {
    fn context(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::SerialisationError(s.to_string(), e.to_string()))
    }
}

impl<T> ResultExt<T> for Result<T, toml::de::Error> {
    fn context(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::DeserialisationError(s.to_string(), e.message().to_string()))
    }
}

impl From<ignore::Error> for Error {
    fn from(e: ignore::Error) -> Self {
        Error::Io("error with exclude flags".into(), e.to_string())
    }
}
