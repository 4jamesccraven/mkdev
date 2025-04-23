use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
pub enum SearchableCommands {
    Evoke,
    Delete,
    List,
}

impl Default for SearchableCommands {
    fn default() -> Self {
        Self::List
    }
}
