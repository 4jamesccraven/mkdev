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
static CONFIG_PATH_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// The directory that data for mkdev is stored in
    pub recipe_dir: Option<PathBuf>,
    /// User defined in-line substitution commands
    pub subs: HashMap<String, String>,
}

impl Config {
    /// Retrieve the config
    pub fn get() -> Result<&'static Config, String> {
        if CONFIG.get().is_none() {
            let config = Config::load()?;
            CONFIG
                .set(config)
                .map_err(|_| "Failed to set CONFIG".to_string())?;
        }

        Ok(CONFIG.get().expect("Should be set"))
    }

    /// Override the default config path.
    /// Note: users can only do this with a temporary CLI flag.
    pub fn override_path(path: PathBuf) {
        CONFIG_PATH_OVERRIDE
            .set(path)
            .expect("Config override already set.");
    }

    /// Private api for loading the config if it is not already loaded.
    /// Reads the file from the default location, or generates a file
    /// if it does not already exist.
    fn load() -> Result<Config, String> {
        // The config file is overridden, or is default
        let config_file = match CONFIG_PATH_OVERRIDE.get() {
            Some(path) => path.clone(),
            None => dirs::config_dir()
                .ok_or("Unable to access user configuration directory: {why}.")?
                .join("mkdev")
                .join("config.toml"),
        };

        // Ensure the parent directory exists
        if let Some(dir) = config_file.parent() {
            if !dir.is_dir() {
                let _ = fs::create_dir_all(&dir).map_err(|why| {
                    format!("Unable to create mkdev configuration directory: {why:?}")
                })?;
            }
        }

        if !config_file.is_file() {
            let cfg = Config::default();
            let serialized_default = toml::to_string(&cfg)
                .expect("Default configuration should always serialize correctly");

            let _ = fs::write(config_file, serialized_default)
                .map_err(|why| format!("Unable to write default configuration file: {why:?}"))?;

            Ok(cfg)
        } else {
            let cfg_contents = fs::read_to_string(config_file)
                .map_err(|why| format!("Unable to read configuration file: {why:?}"))?;

            let cfg: Config = toml::from_str(&cfg_contents).map_err(|_| {
                "Invalid configuration file. Please ensure it is properly formatted.".to_string()
            })?;

            Ok(cfg)
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
