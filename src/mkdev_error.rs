use std::fmt::Display;
use std::io;

#[derive(Clone, Debug)]
pub enum Error {
    NoneSpecified(String),
    Invalid(String, Option<Vec<String>>),
    IoBased(String, String),
    SerialisationError(String, String),
    #[allow(unused)]
    DeserialisationError(String, String),
    DestructionWarning(String),
}

impl Error {
    pub fn from_io(context: &str, why: &io::Error) -> Self {
        Self::IoBased(context.into(), format!("{why}"))
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            NoneSpecified(object) => f.write_fmt(format_args!("no {object} specified.")),
            Invalid(object, examples) => {
                let examples = match examples {
                    Some(eg) => format!(":\n{}", eg.join("\n")),
                    None => String::from("."),
                };

                f.write_fmt(format_args!("invalid {object}{examples}"))
            }
            IoBased(context, why) => f.write_fmt(format_args!("{context}: {why}")),
            SerialisationError(object, why) => {
                f.write_fmt(format_args!("failed to serialise {object}: {why}"))
            }
            DeserialisationError(object, why) => {
                f.write_fmt(format_args!("failed to deserialise {object}: {why}"))
            }
            DestructionWarning(object) => f.write_fmt(format_args!(
                "'{object}' already exists. Use -s to overwrite."
            )),
        }
    }
}
