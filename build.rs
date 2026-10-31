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

fn main() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let manifest: toml::Value = toml::from_str(&manifest).unwrap();

    let license = manifest["package"]["license"].as_str().unwrap();

    println!("cargo:rustc-env=LICENSE_IDENT={license}");
    println!("cargo:rerun-if-changed=locales/");
}
