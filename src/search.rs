use mkdev_cli::searchable::SearchableCommands::{self, *};
use mkdev_recipe::config::Config;

use duct::cmd;

pub fn search(action: Option<SearchableCommands>) -> Result<(), String> {
    let config = Config::get();

    if !config.fzf_integration {
        return Err("fzf_integration is not enabled.".into());
    }

    let action = action.unwrap_or(SearchableCommands::default());

    let selection = cmd!("mk", "list", "--type", "plain")
        .pipe(cmd!("fzf"))
        .read()
        .map_err(|_| "could not perform pipeline from mk -> fzf. Is fzf installed?".to_string())?;

    run_mk(action, selection)?;

    Ok(())
}

fn run_mk(action: SearchableCommands, recipe: String) -> Result<(), String> {
    let action: String = (match action {
        List => "list",
        Delete => "delete",
        Evoke => "evoke",
    })
    .into();

    let _ = cmd!("mk", action, recipe)
        .start()
        .map_err(|_| "Failed to start command.".to_string())?
        .wait()
        .map_err(|_| "Failed to run command.".to_string())?;

    Ok(())
}
