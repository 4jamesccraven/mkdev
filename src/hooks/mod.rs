//! Functions that are tangential to or mutually exclusive with recipe logic.
mod cfg;
mod i18n;
mod man;

use crate::cli::Cli;
use crate::mkdev_error::Error;

/// Calls every mkdev hook sequentially.
pub fn hooks(args: &Cli) -> Result<(), Error> {
    // locale selection
    i18n::hook();

    // mkdev configuration overrides
    cfg::hook(args);

    // man generation
    // note: always exits program.
    man::hook(args)?;

    Ok(())
}
