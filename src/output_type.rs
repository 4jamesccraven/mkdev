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
//! Display formats for `mk list`
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
    Print0,
}
