use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
#[non_exhaustive]
/// The style of output desired by the user. Used by the --type flag
pub enum OutputType {
    Default,
    Debug,
    Plain,
    JSON,
    TOML,
    Nix,
}
