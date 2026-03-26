//! A hook that sets up internationalisation for mkdev.
use rust_i18n::{available_locales, set_locale};
use sys_locale::get_locales as preferred_locales;

pub fn hook() {
    // TODO: properly select locale by finding the first preferred that is available.
    let preferred = preferred_locales().next().unwrap();

    set_locale(&preferred);
}
