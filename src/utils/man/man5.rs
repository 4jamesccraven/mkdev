// mkdev - Save your boilerplate instead of writing it
// Copyright (C) 2026  James C. Craven <4jamesccraven@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Implementation of the renderer for mkdev's config manual.
use crate::config::Config;

use clap::{crate_name, crate_version};
use clap_mangen::roff::{Roff, bold, roman};
use confique::meta::Meta;

/// Generates a man page documenting mkdev's config.
pub fn man5() -> Roff {
    let mut roff = Roff::new();

    man5_header(&mut roff);
    man5_opts(&mut roff, &<Config as confique::Config>::META);
    man5_footer(&mut roff);

    roff
}

fn man5_header(roff: &mut Roff) {
    roff
        // Title Header
        .control(
            "TH",
            [
                "mkdev-config",
                "5",
                // Footer middle
                " ",
                // Footer inside
                concat!(crate_name!(), " ", crate_version!()),
                // Header inside
                "File Formats and Configuration",
            ],
        )
        // Manpage Name
        .control("SH", ["NAME"])
        .text([roman("mkdev-config - Configuration file for mkdev")])
        // Manpage Description
        .control("SH", ["DESCRIPTION"])
        .text([
            bold("mkdev"),
            roman(concat!(
                " stores its configuration file at ~/.config/mkdev/config.toml by default.",
                " This can be overridden with the --config flag or CONFIG environment variable;",
                " see "
            )),
            bold("mk(1)"),
        ])
        // Options stub
        .control("SH", ["CONFIGURATION OPTIONS"])
        .control("SS", ["global options"]);
}

fn man5_footer(roff: &mut Roff) {
    // See also section
    roff.control("SH", ["SEE ALSO"]).text([
        bold("mk(1)"),
        roman(", "),
        bold("mk-evoke(1)"),
        roman(", "),
        bold("mk-list(1)"),
    ]);
}

/// Takes in an unrendered roff file and metadata about the config struct and inserts documentation
/// about that metadata into the roff file. Recurses into metadata about nested structures.
fn man5_opts(roff: &mut Roff, meta_root: &Meta) {
    use confique::meta::FieldKind;
    // Parse metadata out of the struct
    meta_root.fields.iter().for_each(|field| match field.kind {
        FieldKind::Leaf { .. } => {
            let description = field
                .doc
                .iter()
                .map(|s| s.trim())
                .take_while(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            let defaults = field
                .doc
                .iter()
                .map(|s| s.trim())
                .skip_while(|s| !s.starts_with("Default:"))
                .skip(1)
                .collect::<Vec<_>>();

            roff.control("TP", [])
                .text([bold(field.name)])
                .text([roman(description)])
                .control("sp", [])
                .text([bold("Default:")])
                .control("br", []);

            for default in defaults.into_iter() {
                roff.control("br", []).text([roman(default)]);
            }
        }
        FieldKind::Nested { meta, .. } => {
            roff.control("SS", [field.name])
                .text([roman(field.doc.first().map_or("", |s| (*s).trim()))]);
            man5_opts(roff, meta);
        }
    });
}
