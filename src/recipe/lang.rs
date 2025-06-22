use std::fmt::Display;

use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    pub name: String,
    pub colour: Option<(u8, u8, u8)>,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.colour {
            Some(col) => {
                let (r, g, b) = col;
                write!(f, "{}", self.name.truecolor(r, g, b))
            }
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

impl From<&str> for Language {
    fn from(value: &str) -> Self {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^\x1b\[38;2;(\d{1,3});(\d{1,3});(\d{1,3})m(.*?)\x1b\[0m$").unwrap()
        });

        if let Some(caps) = RE.captures(value) {
            if let (Ok(r), Ok(g), Ok(b)) = (caps[1].parse(), caps[2].parse(), caps[3].parse()) {
                return Language {
                    name: caps[4].to_string(),
                    colour: Some((r, g, b)),
                };
            }
        }

        Language {
            name: value.into(),
            colour: None,
        }
    }
}
