use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use dirs;
use serde::{Deserialize, Serialize};
use toml;

// There should only ever be one instance of the config to prevent
// multiple intialisations
static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// The directory that data for mkdev is stored in
    pub recipe_dir: Option<PathBuf>,
    /// User defined in-line substitution commands
    pub subs: HashMap<String, String>,
}

impl Config {
    /// Retrieve the config
    pub fn get() -> &'static Config {
        CONFIG.get_or_init(Config::load)
    }

    /// Private api for loading the config if it is not already loaded.
    /// Reads the file from the default location, or generates a file
    /// if it does not already exist.
    fn load() -> Config {
        let mut config_dir = dirs::config_dir().unwrap_or_else(|| {
            panic!("Unable to access user configuration directory");
        });
        config_dir.push("mkdev");

        if !config_dir.is_dir() {
            let _ = fs::create_dir_all(&config_dir).unwrap_or_else(|error| {
                panic!("Unable to create mkdev configuration directory: {error:?}");
            });
        }

        let config_file = config_dir.join("config.toml");

        if !config_file.is_file() {
            let cfg = Config::default();
            let serialized_default = toml::to_string(&cfg)
                .expect("Default configuration should alway serialize correctly");

            let _ = fs::write(config_file, serialized_default).unwrap_or_else(|error| {
                panic!("Unable to write default configuration file: {error:?}");
            });

            cfg
        } else {
            let cfg_contents = fs::read_to_string(config_file).unwrap_or_else(|error| {
                panic!("Unable to read configuration file: {error:?}");
            });
            let cfg: Config = toml::from_str(&cfg_contents)
                .expect("Invalid configuration file. Please ensure it is properly formatted.");

            cfg
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let recipe_dir: _ = None;

        let subs: HashMap<String, String> = [
            ("dir", "mk::dir"),
            ("user", "whoami"),
            ("day", "date +%d"),
            ("month", "date +%m"),
            ("year", "date +%Y"),
        ]
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .into_iter()
        .collect();

        Self { recipe_dir, subs }
    }
}
