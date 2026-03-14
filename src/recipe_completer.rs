//! An internal function that allows shell completions to detect the user's recipes.
use crate::recipe::Recipe;

use clap_complete::engine::CompletionCandidate;

/// An argument completer for the CLI that returns matching recipe names.
pub fn recipe_completer(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let mut completions = vec![];

    if let Some(current) = current.to_str()
        && let Ok(recipes) = Recipe::gather()
    {
        recipes.iter().map(|r| r.0).for_each(|c: &String| {
            if c.starts_with(current) {
                completions.push(CompletionCandidate::new(c));
            }
        });
    }

    completions
}
