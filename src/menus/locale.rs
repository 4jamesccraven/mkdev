//! Provides locale-aware formatters and parsers to be used with interactive mode menus.
use rust_i18n::t;

/// Locale-aware boolean formatter for Inquire
pub fn locale_bool_formatter(value: bool) -> String {
    match value {
        true => t!("general.yes").to_string(),
        false => t!("general.no").to_string(),
    }
}

/// Locale-aware default boolean ("y/N") formatter for Inquire.
pub fn locale_bool_default_formatter(value: bool) -> String {
    let (yes, no) = (&t!("general.yes_simple"), &t!("general.no_simple"));
    let (mut y, mut n) = (first_char(yes), first_char(no));

    match value {
        true => y = y.to_uppercase().next().unwrap(),
        false => n = n.to_uppercase().next().unwrap(),
    }

    format!("{y}/{n}")
}

/// Locale-aware boolean parser for Inquire.
pub fn locale_bool_parser(input: &str) -> Result<bool, ()> {
    let (yes, no) = (&t!("general.yes_simple"), &t!("general.no_simple"));

    // Get &str of first character of yes and no
    let y_c = first_char(yes);
    let mut y_buf = [0u8; 4];
    let y = y_c.encode_utf8(&mut y_buf);

    let n_c = first_char(no);
    let mut n_buf = [0u8; 4];
    let n = n_c.encode_utf8(&mut n_buf);

    match input {
        input if input == yes => Ok(true),
        input if input == no => Ok(false),
        input if input == y => Ok(true),
        input if input == n => Ok(false),
        _ => Err(()),
    }
}

fn first_char(s: &str) -> char {
    s.chars().next().expect("empty locale string encountered")
}
