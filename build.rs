use std::env;
use std::fs::write;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

use clap::CommandFactory;
use clap_mangen::Man;
use mkdev_cli::cli::Cli;

fn main() -> std::io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").ok_or(NotFound)?;

    let out_dir = PathBuf::from(out_dir);

    let command = Cli::command();

    let man = Man::new(command.clone());
    let mut buffer: Vec<u8> = vec![];

    man.render(&mut buffer)?;

    for subcommand in command.get_subcommands() {
        Man::new(subcommand.clone()).render(&mut buffer)?;
    }

    write(out_dir.join("mk.1"), buffer)?;

    Ok(())
}
