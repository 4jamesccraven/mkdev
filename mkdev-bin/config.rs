use mkdev_cli::cli::Cli;
use mkdev_recipe::config::Config;

/// Hook that handles flags related to configs
pub fn hook(args: &Cli) {
    let skip_main_logic = vec![args.gen_config, args.print_config].iter().any(|f| *f);
    let commands_present = args.command.is_some();

    if skip_main_logic && commands_present {
        eprintln!("mkdev: warning: subcommand suppressed by one or more flags.");
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
        Err(why) => {
            eprintln!("mkdev: error: could not get config {why}.");
            std::process::exit(1);
        }
    };

    let config = match toml::to_string_pretty(&config) {
        Ok(cfg) => cfg,
        Err(_) => {
            eprintln!("Improperly formatted configuration file.");
            std::process::exit(1);
        }
    };

    print!("{config}");
}

fn print_default_config() {
    let config_str = toml::to_string_pretty(&Config::default())
        .expect("Default configuration should alway serialise.");

    print!("{config_str}");
}
