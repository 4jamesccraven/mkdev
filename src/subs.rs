use crate::config::Config;
use crate::warning;

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use regex::Regex;

pub struct Replacer {
    map: HashMap<String, String>,
    re: Regex,
}

impl Replacer {
    pub fn new() -> Self {
        let cfg = Config::get().expect("Config errors should be detected by this point.");

        let map = cfg.subs.clone();

        let combined = map
            .keys()
            // Creates a regular expression that matches {{key1}} or {{key2}} ...
            // The exact regex produced is of form \{\{(key1)\}\}|\{\{(key2)\}\}|...
            // This does two things:
            // 1. Search for "{{key}}"
            // 2. Captures "key" into a group for later usage
            .map(|r| format!(r"\{{\{{({})\}}\}}", r))
            .collect::<Vec<_>>()
            .join("|");

        let re = Regex::new(&combined).unwrap();

        Self { map, re }
    }

    pub fn sub(&self, text: &str, name: &str, dir: &Path) -> String {
        self.re
            .replace_all(text, |caps: &regex::Captures| {
                // Find the group that was matched on
                let match_ = caps
                    .iter()
                    .skip(1)
                    .find_map(|s| s.map(|m| m.as_str()))
                    .unwrap_or("");

                // Set the original string as a fallback
                let fallback = caps.get(0).map_or("", |m| m.as_str());

                match self.map.get(match_) {
                    // Special cases
                    Some(val) if val == "mk::dir" => dir.to_string_lossy().to_string(),
                    Some(val) if val == "mk::name" => name.to_string(),
                    // Substitution in Arbitrary case
                    Some(val) => {
                        // This has to be a closure to take in the environment
                        let sub: Result<String, std::io::Error> = (|| {
                            // Treat the string as a command and try to pass it to the shell
                            let cmd = if cfg!(target_family = "unix") {
                                Command::new("sh").arg("-c").arg(val).output()?
                            } else {
                                // Use cmd instead of sh if not on Unix
                                Command::new("cmd").arg("/C").arg(val).output()?
                            };

                            let output = String::from_utf8_lossy(&cmd.stdout).into_owned();

                            let out = output
                                .strip_suffix("\n")
                                .unwrap_or_else(|| &output)
                                .to_owned();

                            Ok(out)
                        })();

                        sub.unwrap_or_else(|err| {
                            warning!("unable to substitute `{}`: {}", match_, err);
                            fallback.to_string()
                        })
                    }
                    None => fallback.to_string(),
                }
            })
            .to_string()
    }
}
