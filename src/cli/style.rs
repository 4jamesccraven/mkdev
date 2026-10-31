use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Color};

/// Mkdev's accent colour.
///
/// Hex: #F1920D
pub const MKDEV_ACCENT: (u8, u8, u8) = (241, 146, 13);

/// CLI styling for mkdev.
pub fn mkdev_style() -> Styles {
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
