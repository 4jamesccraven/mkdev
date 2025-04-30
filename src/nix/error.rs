use serde::ser;

#[derive(Debug)]
pub enum Error {
    Message(String),
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Message(msg) => f.write_str(&msg),
        }
    }
}

impl std::error::Error for Error {}
