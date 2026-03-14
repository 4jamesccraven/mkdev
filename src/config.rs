//! mkdev's user configuration file.
use crate::display::DisplayConfig;
use crate::mkdev_error::{Error, ResultExt};

use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// There should only ever be one instance of the config to prevent
// multiple intialisations
static CONFIG: OnceLock<Config> = OnceLock::new();
static CONFIG_PATH_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Path to where recipes should be read from/saved to
    /// Default: None (evaluates to ~/.local/share/mkdev on Linux)
    pub recipe_dir: Option<PathBuf>,
    /// User defined variables for recipe building
    /// Default: See `Config::default`
    #[serde(default = "default_subs")]
    pub subs: HashMap<String, String>,
    /// User defined formatting for recipes
    /// Default: See `DisplayConfig::default`
    #[serde(default)]
    pub recipe_fmt: DisplayConfig,
}

impl Config {
    /// Retrieves the user config.
    ///
    /// The config is cached on the first call, so it is safe to call it more than once.
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
    ///
    /// This is used to implement the global `--config` flag.
    pub fn override_path(path: PathBuf) {
        CONFIG_PATH_OVERRIDE
            .set(path)
            .expect("Config override already set.");
    }

    /// Private api for loading the config if it is not already loaded.
    ///
    /// The file is read in from the default location (or the user-provided override), or a default
    /// is provided.
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
        if let Some(dir) = config_file.parent()
            && !dir.is_dir()
        {
            fs::create_dir_all(dir).context("unable to create mkdev configuration directory")?;
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

/// The default substitutions to be used at build time.
///
/// mk::dir and mk::name are special reserved values provided directly by mkdev. The other values
/// are some simple defaults to get the currently logged in user's username or to get the
/// components of the date.
fn default_subs() -> HashMap<String, String> {
    HashMap::from_iter(
        [
            ("dir", "mk::dir"),
            ("name", "mk::name"),
            ("user", "whoami"),
            ("day", "date +%d"),
            ("month", "date +%m"),
            ("year", "date +%Y"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string())),
    )
}

impl Default for Config {
    fn default() -> Self {
        let recipe_dir = None;
        let subs = default_subs();
        let recipe_fmt = DisplayConfig::default();

        Self {
            recipe_dir,
            subs,
            recipe_fmt,
        }
    }
}
