// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Minimal NPY v1 parser for the .npy entries inside lox-itur-data.npz.
//!
//! Spec: <https://numpy.org/doc/stable/reference/generated/numpy.lib.format.html>
//! Supports only what the ITU-R bundle uses: little-endian f64, C order,
//! 1-D or 2-D shape, NPY format version 1.x.

use thiserror::Error;

use crate::grid::RegularGrid2D;

#[derive(Debug, Error)]
pub enum NpyError {
    #[error("not a NPY file (missing magic)")]
    BadMagic,
    #[error("unsupported NPY major version {0}")]
    UnsupportedVersion(u8),
    #[error("malformed NPY header")]
    BadHeader,
    #[error("unsupported dtype {0:?}; expected '<f8'")]
    UnsupportedDtype(String),
    #[error("expected C order (fortran_order: False)")]
    FortranOrder,
    #[error("unsupported shape {0:?}; expected 1-D or 2-D")]
    UnsupportedShape(Vec<usize>),
    #[error("data length mismatch: expected {expected} f64s, got {actual}")]
    LengthMismatch { expected: usize, actual: usize },
}

#[derive(Debug)]
pub struct NpyArray {
    pub shape: Vec<usize>,
    pub data: Vec<f64>,
}

/// Parses a NPY file's bytes into a 2-D `f64` array.
pub fn parse_npy(bytes: &[u8]) -> Result<NpyArray, NpyError> {
    if bytes.len() < 10 || &bytes[..6] != b"\x93NUMPY" {
        return Err(NpyError::BadMagic);
    }
    let major = bytes[6];
    if major != 1 {
        return Err(NpyError::UnsupportedVersion(major));
    }
    let header_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let header_start = 10;
    let header_end = header_start + header_len;
    if bytes.len() < header_end {
        return Err(NpyError::BadHeader);
    }
    let header =
        std::str::from_utf8(&bytes[header_start..header_end]).map_err(|_| NpyError::BadHeader)?;
    let (dtype, shape, fortran_order) = parse_header(header)?;
    if dtype != "<f8" {
        return Err(NpyError::UnsupportedDtype(dtype));
    }
    if fortran_order {
        return Err(NpyError::FortranOrder);
    }
    let expected = shape.iter().product::<usize>();
    let data_bytes = &bytes[header_end..];
    let actual = data_bytes.len() / 8;
    if actual < expected {
        return Err(NpyError::LengthMismatch { expected, actual });
    }
    let data: Vec<f64> = data_bytes[..expected * 8]
        .chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
        .collect();
    Ok(NpyArray { shape, data })
}

/// Builds a `RegularGrid2D` from lat/lon meshgrids and a values array, all as NPY blobs.
///
/// Determines step sizes and row orientation from the lat/lon arrays.
pub fn grid_from_npy(
    lat_npy: &[u8],
    lon_npy: &[u8],
    val_npy: &[u8],
) -> Result<RegularGrid2D, NpyError> {
    let lat = parse_npy(lat_npy)?;
    let lon = parse_npy(lon_npy)?;
    let val = parse_npy(val_npy)?;

    let (rows, cols) = match val.shape.as_slice() {
        [r, c] => (*r, *c),
        [c] => (1, *c),
        other => return Err(NpyError::UnsupportedShape(other.to_vec())),
    };

    let lats_1d: Vec<f64> = if matches!(lat.shape.as_slice(), [_, _]) {
        (0..rows).map(|r| lat.data[r * cols]).collect()
    } else {
        lat.data.clone()
    };
    let lons_1d: Vec<f64> = if matches!(lon.shape.as_slice(), [_, _]) {
        (0..cols).map(|c| lon.data[c]).collect()
    } else {
        lon.data.clone()
    };

    let lat_step_raw = if lats_1d.len() > 1 {
        lats_1d[1] - lats_1d[0]
    } else {
        1.0
    };
    let lon_step = if lons_1d.len() > 1 {
        (lons_1d[1] - lons_1d[0]).abs()
    } else {
        1.0
    };
    let needs_flip = lat_step_raw < 0.0;
    let (lat_start, lat_step) = if needs_flip {
        (*lats_1d.last().unwrap(), lat_step_raw.abs())
    } else {
        (lats_1d[0], lat_step_raw)
    };

    let data: Vec<f64> = if needs_flip {
        let mut out = Vec::with_capacity(rows * cols);
        for r in (0..rows).rev() {
            out.extend_from_slice(&val.data[r * cols..(r + 1) * cols]);
        }
        out
    } else {
        val.data
    };

    Ok(RegularGrid2D::new(
        lat_start, lat_step, rows, lons_1d[0], lon_step, cols, data,
    ))
}

/// Parses the header dict for (`descr`, `shape`, `fortran_order`).
fn parse_header(header: &str) -> Result<(String, Vec<usize>, bool), NpyError> {
    let descr = pick_str(header, "'descr':")
        .or_else(|| pick_str(header, "\"descr\":"))
        .ok_or(NpyError::BadHeader)?;
    let fortran_order =
        if header.contains("'fortran_order': True") || header.contains("\"fortran_order\": True") {
            true
        } else if header.contains("'fortran_order': False")
            || header.contains("\"fortran_order\": False")
        {
            false
        } else {
            return Err(NpyError::BadHeader);
        };
    let shape_start = header
        .find("'shape':")
        .or_else(|| header.find("\"shape\":"))
        .ok_or(NpyError::BadHeader)?;
    let after = &header[shape_start..];
    let lp = after.find('(').ok_or(NpyError::BadHeader)?;
    let rp = after.find(')').ok_or(NpyError::BadHeader)?;
    let inner = &after[lp + 1..rp];
    let shape: Vec<usize> = inner
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().map_err(|_| NpyError::BadHeader))
        .collect::<Result<_, _>>()?;
    Ok((descr, shape, fortran_order))
}

fn pick_str(header: &str, key: &str) -> Option<String> {
    let start = header.find(key)? + key.len();
    let rest = &header[start..];
    let q1 = rest.find('\'').or_else(|| rest.find('"'))?;
    let after = &rest[q1 + 1..];
    let q2 = after.find('\'').or_else(|| after.find('"'))?;
    Some(after[..q2].to_owned())
}

/// Build a v1 NPY blob for a 2-D f64 array. Layout: magic + version + u16 header_len + header + data.
#[cfg(test)]
pub(crate) fn tests_synth_npy_2d(rows: usize, cols: usize, data: &[f64]) -> Vec<u8> {
    assert_eq!(data.len(), rows * cols);
    let header_str =
        format!("{{'descr': '<f8', 'fortran_order': False, 'shape': ({rows}, {cols}), }}");
    // pad with spaces + newline so (10 + header_len) is a multiple of 64
    let unpadded = 10 + header_str.len() + 1;
    let pad = (64 - unpadded % 64) % 64;
    let mut header = header_str.into_bytes();
    header.extend(std::iter::repeat_n(b' ', pad));
    header.push(b'\n');
    let mut out = Vec::with_capacity(10 + header.len() + data.len() * 8);
    out.extend_from_slice(b"\x93NUMPY");
    out.push(1);
    out.push(0);
    out.extend_from_slice(&(header.len() as u16).to_le_bytes());
    out.extend_from_slice(&header);
    for v in data {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_2d() {
        let blob = tests_synth_npy_2d(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let arr = parse_npy(&blob).unwrap();
        assert_eq!(arr.shape, vec![2, 3]);
        assert_eq!(arr.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn rejects_bad_magic() {
        assert!(matches!(parse_npy(b"NOTANPY___"), Err(NpyError::BadMagic)));
    }

    #[test]
    fn grid_from_npy_no_flip() {
        let lat = tests_synth_npy_2d(
            3,
            4,
            &[
                -10.0, -10.0, -10.0, -10.0, 0.0, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0,
            ],
        );
        let lon = tests_synth_npy_2d(
            3,
            4,
            &[
                0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0,
            ],
        );
        let val = tests_synth_npy_2d(
            3,
            4,
            &[
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
        );
        let g = grid_from_npy(&lat, &lon, &val).unwrap();
        // grid point at lat=0, lon=10 should be 6.0
        assert!((g.bilinear(0.0, 10.0) - 6.0).abs() < 1e-12);
    }

    #[test]
    fn grid_from_npy_with_flip() {
        // lat axis runs N→S (decreasing); parser should flip rows so lookup is consistent
        let lat = tests_synth_npy_2d(
            3,
            4,
            &[
                10.0, 10.0, 10.0, 10.0, 0.0, 0.0, 0.0, 0.0, -10.0, -10.0, -10.0, -10.0,
            ],
        );
        let lon = tests_synth_npy_2d(
            3,
            4,
            &[
                0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0, 0.0, 10.0, 20.0, 30.0,
            ],
        );
        // values in N→S order
        let val = tests_synth_npy_2d(
            3,
            4,
            &[
                9.0, 10.0, 11.0, 12.0, 5.0, 6.0, 7.0, 8.0, 1.0, 2.0, 3.0, 4.0,
            ],
        );
        let g = grid_from_npy(&lat, &lon, &val).unwrap();
        assert!((g.bilinear(0.0, 10.0) - 6.0).abs() < 1e-12);
        assert!((g.bilinear(-10.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((g.bilinear(10.0, 30.0) - 12.0).abs() < 1e-12);
    }
}
