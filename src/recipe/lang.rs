//! Data type that represents a programming language or file format.
//!
//! Interfaces with hyperpolyglot to store the name and colour of a programming language.
use std::fmt::Display;

use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    pub name: String,
    pub colour: Option<(u8, u8, u8)>,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.colour {
            Some((r, g, b)) => write!(f, "{}", self.name.truecolor(r, g, b)),
            None => write!(f, "{}", self.name),
        }
    }
}

impl From<hyperpolyglot::Language> for Language {
    fn from(value: hyperpolyglot::Language) -> Self {
        let name = value.name.to_string();

        let colour: Option<(u8, u8, u8)> = value.color.and_then(|s| {
            let hex = &s[1..].to_string();

            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

            Some((r, g, b))
        });

        Language { name, colour }
    }
}

/// Backwards compatibility for V1 Language Strings.
///
/// This `From` implementation attempts to parse out a colour if it can, or just uses the given
/// string as a name and sets colour to none if it can't.
impl From<&str> for Language {
    fn from(value: &str) -> Self {
        let mut name = value;
        let colour: Option<(u8, u8, u8)> = (|| {
            let s = value.strip_prefix("\x1b[38;2;")?;

            let (r, s) = s.split_once(';')?;
            let (g, s) = s.split_once(';')?;
            let (b, s) = s.split_once('m')?;

            let r = r.parse().ok()?;
            let g = g.parse().ok()?;
            let b = b.parse().ok()?;

            name = s.strip_suffix("\x1b[0m")?;

            Some((r, g, b))
        })();
        let name = name.to_string();

        Language { name, colour }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_ansi_string() {
        let lang = Language::from("\x1b[38;2;255;100;0mRust\x1b[0m");
        assert_eq!(lang.name, "Rust");
        assert_eq!(lang.colour, Some((255, 100, 0)));
    }

    #[test]
    fn test_language_from_plain_string() {
        let lang = Language::from("Rust");
        assert_eq!(lang.name, "Rust");
        assert_eq!(lang.colour, None);
    }

    #[test]
    fn test_language_from_malformed_ansi_string() {
        let lang = Language::from("\x1b[38;2;999;0;0mRust\x1b[0m");
        assert_eq!(lang.name, "\x1b[38;2;999;0;0mRust\x1b[0m");
        assert_eq!(lang.colour, None);
    }
}
