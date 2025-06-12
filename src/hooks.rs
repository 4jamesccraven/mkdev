//! Functions that "hook" into the beginning of main, potentially allowing short circuiting of any
//! other logic that mkdev might do.
use crate::cli::Cli;
use crate::config::Config;
use crate::{die, warning};

use std::io::Write;

use clap::CommandFactory;
use clap_mangen::Man;

pub fn hooks(args: &Cli) {
    config(args);
    man(args);
}

//=Config=//

/// Hook that handles flags related to configs
fn config(args: &Cli) {
    let skip_main_logic = vec![args.gen_config, args.print_config].iter().any(|f| *f);
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

fn print_default_config() {
    let config_str = toml::to_string_pretty(&Config::default())
        .expect("Default configuration should alway serialise.");

    print!("{config_str}");
}

//=man page=//

fn man(args: &Cli) {
    if args.man_page {
        let command = Cli::command();
        let man = Man::new(command.clone());
        let mut output_buffer: Vec<u8> = vec![];

        man.render(&mut output_buffer)
            .expect("Writing to Vec<u8> is infallible.");

        for subcommand in command.get_subcommands() {
            Man::new(subcommand.clone())
                .render(&mut output_buffer)
                .expect("Writing to Vec<u8> is infallible.");
        }

        std::io::stdout()
            .lock()
            .write(&output_buffer)
            .expect("Unable to write to stdout");

        std::process::exit(0);
    }
}
