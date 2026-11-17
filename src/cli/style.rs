use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Color};
use indicatif::ProgressStyle;

/// Mkdev's accent colour.
// RGB
pub const MKDEV_ACCENT: (u8, u8, u8) = (241, 146, 13);
// HEX
#[rustfmt::skip]
macro_rules! mkdev_accent_hex { () => { "#F1920D" }; }
pub const MKDEV_ACCENT_HEX: &str = mkdev_accent_hex!();

/// CLI styling for mkdev.
pub fn mkdev_cli_style() -> Styles {
    let accent = Color::from(MKDEV_ACCENT).on_default();

    Styles::styled()
        .header(accent.underline())
        .usage(accent)
        // flags, aliases, and environment variables
        .literal(AnsiColor::Green.on_default())
        .context_value(AnsiColor::Green.on_default())
        // Placeholder names (e.g. <RECIPE>) and env/alias boxes
        .placeholder(AnsiColor::Yellow.on_default())
        .context(AnsiColor::Yellow.on_default())
}

/// Progress bar styling for mkdev.
pub fn mkdev_pb_style() -> ProgressStyle {
    ProgressStyle::with_template(concat!(
        "[{bar:40.",
        mkdev_accent_hex!(),
        "/black}] ",
        "{pos:>4}/{len:4} {wide_msg}"
    ))
    .expect("pb template error")
    .progress_chars("=>-")
}
