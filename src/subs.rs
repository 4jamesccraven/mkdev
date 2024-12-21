use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{Datelike, Local};
use regex::Regex;
use whoami::username;

pub struct Replacer {
    map: HashMap<String, Box<dyn Fn(&PathBuf) -> String>>,
}

impl Replacer {
    pub fn new() -> Self {
        let map = [
            (
                "dir",
                Box::new(|dir: &PathBuf| dir.to_string_lossy().to_string())
                    as Box<dyn Fn(&PathBuf) -> String>,
            ),
            (
                "user",
                Box::new(|_dir: &PathBuf| username()) as Box<dyn Fn(&PathBuf) -> String>,
            ),
            (
                "day",
                Box::new(|_dir: &PathBuf| {
                    let now = Local::now();
                    now.day().to_string()
                }) as Box<dyn Fn(&PathBuf) -> String>,
            ),
            (
                "month",
                Box::new(|_dir: &PathBuf| {
                    let now = Local::now();
                    now.month().to_string()
                }) as Box<dyn Fn(&PathBuf) -> String>,
            ),
            (
                "year",
                Box::new(|_dir: &PathBuf| {
                    let now = Local::now();
                    now.year().to_string()
                }) as Box<dyn Fn(&PathBuf) -> String>,
            ),
            (
                "weekday",
                Box::new(|_dir: &PathBuf| {
                    let now = Local::now();
                    now.weekday().to_string()
                }) as Box<dyn Fn(&PathBuf) -> String>,
            ),
        ]
        .map(|(r, c)| (r.to_string(), c))
        .into_iter()
        .collect();

        Self { map }
    }

    pub fn sub(&self, text: &str, dir: &PathBuf) -> String {
        let combined = self
            .map
            .keys()
            .map(|r| {
                let pat = format!(r"\{{\{{({})\}}\}}", r);
                pat
            })
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join("|");

        let re = Regex::new(&combined).unwrap();

        re.replace_all(text, |caps: &regex::Captures| {
            let mut fallback = "";
            let mut match_ = "";

            for (i, s) in caps.iter().enumerate() {
                match i {
                    0 => fallback = s.unwrap().as_str(),
                    _ => {
                        if let Some(val) = s {
                            match_ = val.as_str();
                            break;
                        }
                    }
                }
            }

            if let Some(val) = self.map.get(match_) {
                val(dir)
            } else {
                fallback.to_string()
            }
        })
        .to_string()
    }
}
