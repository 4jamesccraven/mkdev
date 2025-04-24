use crate::cli::Cli;

use std::io::Write;

use clap::CommandFactory;
use clap_mangen::Man;

pub fn hook(args: &Cli) {
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
