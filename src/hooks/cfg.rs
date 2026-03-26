//! Hook that does basic config operations such as printing, overrides, etc.
use crate::cli::Cli;
use crate::config::Config;
use crate::{die, warning};

/// Handles operations that modify or pertain to mkdev's config file.
///
/// The print operations cause the program to exit early.
pub fn hook(args: &Cli) {
    let skip_main_logic = [args.gen_config, args.print_config].iter().any(|f| *f);
    let commands_present = args.command.is_some();

    if skip_main_logic && commands_present {
        warning!("subcommand suppressed by one or more flags.");
    }

    if args.gen_config {
        print_default_config();
    }

    if let Some(path) = args.config.clone() {
        Config::override_path(path);
    }

    if args.print_config {
        print_config();
    }

    if skip_main_logic {
        std::process::exit(0);
    }
}

/// Deserialises and prints the user's current config.
fn print_config() {
    let config = match Config::get() {
        Ok(config) => config,
        Err(why) => die!("could not get config: {}", why),
    };

    let config = match toml::to_string_pretty(&config) {
        Ok(cfg) => cfg,
        Err(_) => die!("improperly formatted configuration file."),
    };

    print!("{config}");
}

/// Prints the default config to stdout.
///
/// Can be used to reset user config to default.
fn print_default_config() {
    let config_str = toml::to_string_pretty(&Config::default())
        .expect("Default configuration should alway serialise.");

    print!("{config_str}");
}
