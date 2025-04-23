use std::path::PathBuf;

use mkdev_cli::cli::Cli;
use mkdev_recipe::config::Config;

/// Hook that handles flags related to configs
pub fn config_hook(args: &Cli) {
    if args.gen_config {
        let config_str = toml::to_string_pretty(&Config::default())
            .expect("Default configuration should alway serialise.");

        if args.command.is_some() {
            eprintln!("mkdev: warning: subcommand suppressed by `--gen-config` flag.");
        }

        print!("{config_str}");

        std::process::exit(0);
    }

    if let Some(path) = args.config.clone() {
        Config::override_path(path);
    } else if let Ok(dir) = std::env::var("CONFIG") {
        let path = PathBuf::from(dir);

        if path.is_file() {
            Config::override_path(path);
        } else {
            eprintln!("mkdev: warning: `CONFIG` environment variable detected, but path is invalid. Using default.")
        }
    }

    if args.print_config {
        let config = toml::to_string_pretty(&Config::get())
            .expect("Improperly formatted configuration file.");

        if args.command.is_some() {
            eprintln!("mkdev: warning: subcommand suppressed by `--print-config` flag.");
        }

        print!("{config}");

        std::process::exit(0);
    }
}
