//! Functions that "hook" into the beginning of main, potentially allowing short circuiting of any
//! other logic that mkdev might do.
use crate::cli::Cli;
use crate::config::Config;
use crate::mkdev_error::{Error, ResultExt};
use crate::{die, warning};

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use clap::CommandFactory;
use clap_mangen::Man;

/// Call all of Mkdev's hooks
pub fn hooks(args: &Cli) -> Result<(), Error> {
    config(args);
    man(args)?;

    Ok(())
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

fn man(args: &Cli) -> Result<(), Error> {
    if args.man_page {
        let command = Cli::command();

        let out_dir = Path::new("mkdev-man");
        std::fs::create_dir_all(&out_dir).context("unable to make directory for man pages")?;

        // Get all commands as a Vec<Command>
        let to_render: Vec<(clap::Command, Option<String>)> = vec![(command.clone(), None)]
            .into_iter()
            .chain(
                command
                    .get_subcommands()
                    .map(|sc| (sc.to_owned(), Some("mk".to_string()))),
            )
            .collect();

        to_render
            .into_iter()
            .try_for_each(|(command, name)| -> Result<(), Error> {
                // Create a manpage renderer from the command
                let man = Man::new(command.clone());

                // Calculate the filename, and join it into the directory
                let base_filename = man.get_filename();

                let filename = match name {
                    Some(n) => format!("{n}-{base_filename}"),
                    None => base_filename,
                };

                let path = out_dir.join(&filename);

                // Create the file and open it for writing
                let file =
                    File::create(path).context(&format!("unable to create {}", &filename))?;
                let mut writer = BufWriter::new(file);

                // Write the contents of the page into the file
                man.render(&mut writer)
                    .context(&format!("unable to write {}", &filename))?;

                Ok(())
            })?;

        std::process::exit(0);
    }

    Ok(())
}
