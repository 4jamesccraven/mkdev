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

//! Wrappers for simple operations involving gzip (provided by [`flate2`]).
use std::io::{Read, Write};

use crate::error::Error;

use flate2::Compression;
use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;

/// Encodes the provided bytes as gzipped data.
///
/// Returns the encoded bytes, or a [`Error::CompressionFailure`].
pub fn encode_bytes(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).map_err(Error::CompressionFailure)?;
    encoder.finish().map_err(Error::CompressionFailure)
}

/// Decodes the provided gzip data as bytes.
///
/// Returns the decoded bytes, or a [`Error::CompressionFailure`].
pub fn decode_bytes(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decoder = GzDecoder::new(data);
    let mut out = Vec::new();
    _ = decoder
        .read_to_end(&mut out)
        .map_err(Error::CompressionFailure)?;
    Ok(out)
}
