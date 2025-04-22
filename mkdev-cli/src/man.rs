use crate::cli::Cli;

use std::{env, io::Write};

use clap::CommandFactory;
use clap_mangen::Man;

pub fn man_env() {
    if let Ok(val) = env::var("MAN_PAGE") {
        if val == "1" {
            let command = Cli::command();
            let man = Man::new(command.clone());
            let mut output_buffer: Vec<u8> = vec![];

            man.render(&mut output_buffer)
                .expect("TODO: Why might this break?");

            for subcommand in command.get_subcommands() {
                Man::new(subcommand.clone())
                    .render(&mut output_buffer)
                    .expect("TODO: Why might this break?");
            }

            std::io::stdout()
                .lock()
                .write(&output_buffer)
                .expect("Unable to write to stdout");

            std::process::exit(0);
        }
    }
}
