// mkdev - Save your boilerplate instead of writing it
// Copyright (C) 2026  James C. Craven <4jamesccraven@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//! Hook that does basic config operations such as printing, overrides, etc.
use crate::cli::Cli;
use crate::config::Config;
use crate::{die, warning};

use rust_i18n::t;

/// Handles operations that modify or pertain to mkdev's config file.
///
/// The print operations cause the program to exit early.
pub fn hook(args: &Cli) {
    let skip_main_logic = args.gen_config || args.print_config;
    let commands_present = args.command.is_some();

    if skip_main_logic && commands_present {
        warning!("{}", t!("warnings.subcommand_suppressed"));
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
        Err(why) => die!("{}", why),
    };

    let config = toml::to_string_pretty(&config).unwrap();

    print!("{config}");
}

/// Prints the default config to stdout.
///
/// Can be used to reset user config to default.
fn print_default_config() {
    let config_str = toml::to_string_pretty(&Config::default())
        .expect("default `Config` is always serialisable.");

    print!("{config_str}");
}
