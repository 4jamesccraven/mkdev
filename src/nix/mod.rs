mod error;
mod map;
mod seq;
mod ser;
mod r#struct;
mod test;
mod tuple;

pub use error::Error;
use ser::Serializer;

use serde::Serialize;

/// Attempt to serialise to a string, returning an error on failure
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
        pending_key: None,
        indent_depth: 0,
    };
    value.serialize(&mut serializer)?;

    Ok(post_processor(&serializer.output))
}

fn post_processor(serialized: &String) -> String {
    serialized
        .lines()
        .map(|l| if l.chars().any(|c| c != ' ') { l } else { "" })
        .collect::<Vec<_>>()
        .join("\n")
}
