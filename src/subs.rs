use crate::config::Config;

use std::collections::HashMap;
use std::process::Command;
use std::path::PathBuf;

use regex::Regex;

pub struct Replacer {
    map: HashMap<String, String>,
}

impl Replacer {
    pub fn new() -> Self {
        let cfg = Config::get();
        let map = cfg.subs.clone();

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
                        }
                    }
                }
            }

            if let Some(val) = self.map.get(match_) {
                // Special case
                if val == "mk::dir" {
                    return dir.to_string_lossy().to_string();
                }

                let mut parsed = val.split_whitespace().peekable();

                if parsed.peek().is_none() {
                    fallback.to_string()
                } else {
                    let mut cmd = Command::new(&parsed.next().unwrap());
                    cmd.args(parsed);

                    if let Ok(out) = cmd.output() {
                        String::from_utf8_lossy(&out.stdout)
                            .to_string()
                            .strip_suffix("\n")
                            .unwrap()
                            .to_owned()
                    } else {
                        eprintln!("Warning: command `{val}` failed");
                        fallback.to_string()
                    }
                }
            } else {
                fallback.to_string()
            }
        })
        .to_string()
    }
}
