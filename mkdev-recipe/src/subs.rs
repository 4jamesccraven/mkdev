use crate::config::Config;

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

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
            .map(|r| format!(r"\{{\{{({})\}}\}}", r))
            .collect::<Vec<_>>()
            .join("|");

        let re = Regex::new(&combined).unwrap();

        re.replace_all(text, |caps: &regex::Captures| {
            // Find the group that was matched on
            let match_ = caps
                .iter()
                .skip(1)
                .find_map(|s| s.map(|m| m.as_str()))
                .unwrap_or("");

            // Set the original string as a fallback
            let fallback = caps.get(0).map_or("", |m| m.as_str());

            match self.map.get(match_) {
                // Special case
                Some(val) if val == "mk::dir" => dir.to_string_lossy().to_string(),
                // Substitution in Arbitrary case
                Some(val) => {
                    // This has to be a closure to take in the environment
                    let sub: Result<String, &'static str> = (|| {
                        let utf8_err: &'static str = "Unable to get command output";

                        // Treat the string as a command and try to pass it
                        let cmd = if cfg!(target_family = "unix") {
                            Command::new("sh")
                                .arg("-c")
                                .arg(&val)
                                .output()
                                .map_err(|_| utf8_err)?
                        } else {
                            // Use cmd instead of sh if not on Unix
                            Command::new("cmd")
                                .arg("/C")
                                .arg(&val)
                                .output()
                                .map_err(|_| utf8_err)?
                        };

                        let output = String::from_utf8_lossy(&cmd.stdout).into_owned();

                        let out = output
                            .strip_suffix("\n")
                            .unwrap_or_else(|| &output)
                            .to_owned();

                        Ok(out)
                    })();

                    sub.unwrap_or_else(|err| {
                        eprintln!("Unable to substitute `{}`: {}", match_, err);
                        fallback.to_string()
                    })
                }
                None => fallback.to_string(),
            }
        })
        .to_string()
    }
}
