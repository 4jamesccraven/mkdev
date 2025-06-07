use std::io;

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
    IoBased(String, String),

    #[error("failed to serialise {0}: {1}")]
    SerialisationError(String, String),

    #[allow(unused)]
    #[error("failed to deserialise {0}: {1}")]
    DeserialisationError(String, String),

    #[error("'{0}' already exists. Use -s to overwrite.")]
    DestructionWarning(String),
}

pub trait ResultExt<T> {
    fn context(self, s: &str) -> Result<T, Error>;
}

impl<T> ResultExt<T> for Result<T, io::Error> {
    fn context(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::IoBased(s.to_string(), e.to_string()))
    }
}

impl<T> ResultExt<T> for Result<T, ser_nix::Error> {
    fn context(self, s: &str) -> Result<T, Error> {
        self.map_err(|e| Error::SerialisationError(s.to_string(), e.to_string()))
    }
}
