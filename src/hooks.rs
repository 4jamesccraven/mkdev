//! Functions that are tangential to or mutually exclusive with recipe logic.
use crate::cli::Cli;
use crate::config::Config;
use crate::mkdev_error::{Error, ResultExt};
use crate::{die, warning};

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use clap::CommandFactory;
use clap::{crate_name, crate_version};
use clap_mangen::Man;
use clap_mangen::roff::{Roff, bold, roman};
use confique::meta::Meta;

/// Calls every mkdev hook sequentially.
pub fn hooks(args: &Cli) -> Result<(), Error> {
    config(args);
    man(args)?;

    Ok(())
}

// --- Config Hook ---

/// Handles operations that modify or pertain to mkdev's config file.
///
/// The print operations cause the program to exit early.
fn config(args: &Cli) {
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

// --- Manpage Hook ---

/// Generates all of mkdev's man pages and saves them to './mkdev-man'.
fn man(args: &Cli) -> Result<(), Error> {
    // if args.man_page {
    if args.man_page {
        let command = Cli::command();

        let out_dir = Path::new("mkdev-man");
        std::fs::create_dir_all(out_dir).context("unable to make directory for man pages")?;

        // Get all commands as a Vec<Command>
        let to_render: Vec<(clap::Command, Option<&str>)> = vec![(command.clone(), None)]
            .into_iter()
            .chain(
                command
                    .get_subcommands()
                    .map(|sc| (sc.to_owned(), Some("mk"))),
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

        man5()?;

        std::process::exit(0);
    }

    Ok(())
}

// Generates the man page for mkdev's configuration file: `mkdev-config(5)`
fn man5() -> Result<(), Error> {
    let mut roff = Roff::new();

    roff
        // Title Header
        .control(
            "TH",
            [
                "mkdev-config",
                "5",
                // Footer middle
                " ",
                // Footer inside
                concat!(crate_name!(), " ", crate_version!()),
                // Header inside
                "File Formats and Configuration",
            ],
        )
        // Manpage Name
        .control("SH", ["NAME"])
        .text([roman("mkdev-config - Configuration file for mkdev")])
        // Manpage Description
        .control("SH", ["DESCRIPTION"])
        .text([
            bold("mkdev"),
            roman(concat!(
                " stores its configuration file at ~/.config/mkdev/config.toml by default.",
                " This can be overridden with the --config flag or CONFIG environment variable;",
                " see "
            )),
            bold("mk(1)"),
        ])
        // Options stub
        .control("SH", ["CONFIGURATION OPTIONS"])
        .control("SS", ["global options"]);

    // Insert options parsed from metadata.
    insert_opts(&mut roff, &<Config as confique::Config>::META);

    // See also section
    roff.control("SH", ["SEE ALSO"]).text([
        bold("mk(1)"),
        roman(", "),
        bold("mk-evoke(1)"),
        roman(", "),
        bold("mk-list(1)"),
    ]);

    // Create the man page file.
    let man5_file = File::create("mkdev-man/mkdev-config.5")
        .context("unable to create mkdev-man/mkdev-config.5")?;

    // Create a write buffer.
    let mut w = BufWriter::new(man5_file);

    // Render our manpage to it.
    roff.to_writer(&mut w)
        .context("unable to write to mkdev-man/mkdev-config.5")?;

    Ok(())
}

/// Takes in an unrendered roff file and metadata about the config struct and inserts documentation
/// about that metadata into the roff file. Recurses into metadata about nested structures.
fn insert_opts(roff: &mut Roff, meta_root: &Meta) {
    use confique::meta::FieldKind;
    // Parse metadata out of the struct
    meta_root.fields.iter().for_each(|field| match field.kind {
        FieldKind::Leaf { .. } => {
            let description = field
                .doc
                .iter()
                .map(|s| s.trim())
                .take_while(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            let defaults = field
                .doc
                .iter()
                .map(|s| s.trim())
                .skip_while(|s| !s.starts_with("Default:"))
                .skip(1)
                .collect::<Vec<_>>();

            roff.control("TP", [])
                .text([bold(field.name)])
                .text([roman(description)])
                .control("sp", [])
                .text([bold("Default:")])
                .control("br", []);

            for default in defaults.into_iter() {
                roff.control("br", []).text([roman(default)]);
            }
        }
        FieldKind::Nested { meta, .. } => {
            roff.control("SS", [field.name])
                .text([roman(field.doc.get(0).map_or("", |s| (*s).trim()))]);
            insert_opts(roff, meta);
        }
    });
}
