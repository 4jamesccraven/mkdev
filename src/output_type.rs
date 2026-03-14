use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum, Default)]
#[non_exhaustive]
/// The style of output desired by the user. Used by the --type flag
pub enum OutputType {
    #[default]
    Default,
    Debug,
    Plain,
    Json,
    Toml,
    Nix,
}
