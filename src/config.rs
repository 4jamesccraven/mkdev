use crate::mkdev_error::{Error, ResultExt};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// The directory that data for mkdev is stored in
    pub recipe_dir: Option<PathBuf>,
    /// User defined in-line substitution commands
    pub subs: HashMap<String, String>,
}

impl Config {
    /// Retrieve the config
    pub fn get() -> Result<&'static Config, Error> {
        if CONFIG.get().is_none() {
            let config = Config::load()?;
            CONFIG
                .set(config)
                .expect("This block only happens if it is not set.");
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
    fn load() -> Result<Config, Error> {
        // The config file is overridden, or is default
        let config_file = match CONFIG_PATH_OVERRIDE.get() {
            Some(path) => path.clone(),
            None => dirs::config_dir()
                .expect("This is generally infallible")
                .join("mkdev")
                .join("config.toml"),
        };

        // Ensure the parent directory exists
        if let Some(dir) = config_file.parent() {
            if !dir.is_dir() {
                fs::create_dir_all(&dir)
                    .context("unable to create mkdev configuration directory")?;
            }
        }

        if !config_file.is_file() {
            let cfg = Config::default();
            let serialized_default = toml::to_string(&cfg)
                .expect("Default configuration should always serialize correctly");

            fs::write(config_file, serialized_default)
                .context("unable to write default configuration file")?;

            Ok(cfg)
        } else {
            let cfg_contents =
                fs::read_to_string(config_file).context("unable to read configuration file")?;

            let cfg: Config = toml::from_str(&cfg_contents).context("configuration file")?;

            Ok(cfg)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let recipe_dir: _ = None;
        #[rustfmt::skip]
        let default_subs = if cfg!(target_family = "unix") {
            [
                ("dir", "mk::dir"),
                ("name", "mk::name"),
                ("user", "whoami"),
                ("day", "date +%d"),
                ("month", "date +%m"),
                ("year", "date +%Y"),
            ]
        } else {
            [
                ("dir", "mk::dir"),
                ("name", "mk::name"),
                ("user", "whoami"),
                ("day", "for /f \"tokens=2 delims=/\" %a in ('date /t') do @echo %a"),
                ("month", "for /f \"tokens=1 delims=/\" %a in ('date /t') do @echo %a"),
                ("year", "for /f \"tokens=3 delims=/\" %a in ('date /t') do @echo %a"),
            ]
        };

        let subs: HashMap<String, String> = default_subs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .into_iter()
            .collect();

        Self { recipe_dir, subs }
    }
}
