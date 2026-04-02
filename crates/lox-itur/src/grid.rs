// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Regular 2D grid interpolation for ITU-R geophysical data (P.1144).
//!
//! All ITU-R gridded datasets are regular latitude/longitude grids with uniform spacing.
//! This module provides bilinear and nearest-neighbour interpolation on such grids
//! using direct index arithmetic (no search required).

use std::sync::Arc;

/// A regular 2D grid of `f64` values on a latitude/longitude domain.
///
/// Latitude increases with row index (south-to-north convention). Longitude increases
/// with column index. Both axes have uniform spacing.
///
/// Data is stored in row-major order: `data[lat_idx * lon_count + lon_idx]`.
#[derive(Clone, Debug)]
pub struct RegularGrid2D {
    lat_start: f64,
    lat_step: f64,
    lat_count: usize,
    lon_start: f64,
    lon_step: f64,
    lon_count: usize,
    data: Arc<[f64]>,
}

impl RegularGrid2D {
    /// Creates a new regular grid.
    ///
    /// # Arguments
    ///
    /// * `lat_start` — Latitude of the first row (southernmost), in degrees.
    /// * `lat_step` — Latitude spacing between rows, in degrees (positive = northward).
    /// * `lat_count` — Number of latitude rows.
    /// * `lon_start` — Longitude of the first column, in degrees.
    /// * `lon_step` — Longitude spacing between columns, in degrees (positive = eastward).
    /// * `lon_count` — Number of longitude columns.
    /// * `data` — Row-major data, length must equal `lat_count * lon_count`.
    ///
    /// # Panics
    ///
    /// Panics if `data.len() != lat_count * lon_count` or if step sizes are not positive.
    pub fn new(
        lat_start: f64,
        lat_step: f64,
        lat_count: usize,
        lon_start: f64,
        lon_step: f64,
        lon_count: usize,
        data: Vec<f64>,
    ) -> Self {
        assert!(lat_step > 0.0, "lat_step must be positive (south-to-north)");
        assert!(lon_step > 0.0, "lon_step must be positive (west-to-east)");
        assert_eq!(
            data.len(),
            lat_count * lon_count,
            "data length ({}) must equal lat_count * lon_count ({})",
            data.len(),
            lat_count * lon_count,
        );
        Self {
            lat_start,
            lat_step,
            lat_count,
            lon_start,
            lon_step,
            lon_count,
            data: data.into(),
        }
    }

    /// Returns the latitude of the last row (northernmost).
    pub fn lat_end(&self) -> f64 {
        self.lat_start + self.lat_step * (self.lat_count - 1) as f64
    }

    /// Returns the longitude of the last column.
    #[allow(dead_code)]
    pub fn lon_end(&self) -> f64 {
        self.lon_start + self.lon_step * (self.lon_count - 1) as f64
    }

    /// Bilinear interpolation at the given latitude and longitude (both in degrees).
    ///
    /// Latitude is clamped to the grid range. Longitude wraps around.
    pub fn bilinear(&self, lat: f64, lon: f64) -> f64 {
        let (ri, ci) = self.fractional_indices(lat, lon);

        let r0 = ri.floor() as usize;
        let c0 = ci.floor() as usize;
        let r1 = (r0 + 1).min(self.lat_count - 1);
        let c1_raw = c0 + 1;

        let dr = ri - r0 as f64;
        let dc = ci - c0 as f64;

        let v00 = self.get(r0, c0);
        let v01 = self.get(r0, c1_raw);
        let v10 = self.get(r1, c0);
        let v11 = self.get(r1, c1_raw);

        v00 * (1.0 - dr) * (1.0 - dc)
            + v01 * (1.0 - dr) * dc
            + v10 * dr * (1.0 - dc)
            + v11 * dr * dc
    }

    /// Nearest-neighbour interpolation at the given latitude and longitude (degrees).
    ///
    /// Latitude is clamped to the grid range. Longitude wraps around.
    #[allow(dead_code)]
    pub fn nearest(&self, lat: f64, lon: f64) -> f64 {
        let (ri, ci) = self.fractional_indices(lat, lon);
        let r = ri.round() as usize;
        let c = ci.round() as usize;
        let r = r.min(self.lat_count - 1);
        self.get(r, c)
    }

    /// Computes fractional row/column indices for a given (lat, lon).
    ///
    /// Latitude is clamped; longitude wraps modulo the grid span.
    fn fractional_indices(&self, lat: f64, lon: f64) -> (f64, f64) {
        // Clamp latitude
        let lat_end = self.lat_end();
        let lat_clamped = lat.clamp(self.lat_start, lat_end);
        let ri = (lat_clamped - self.lat_start) / self.lat_step;
        // Clamp to valid interpolation range
        let ri = ri.clamp(0.0, (self.lat_count - 1) as f64);

        // Wrap longitude into the grid domain
        let lon_span = self.lon_step * self.lon_count as f64;
        let mut lon_norm = (lon - self.lon_start) % lon_span;
        if lon_norm < 0.0 {
            lon_norm += lon_span;
        }
        let ci = lon_norm / self.lon_step;

        (ri, ci)
    }

    /// Gets a value from the grid, wrapping the column index for longitude periodicity.
    fn get(&self, row: usize, col: usize) -> f64 {
        let col = col % self.lon_count;
        self.data[row * self.lon_count + col]
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use super::*;

    fn sample_grid() -> RegularGrid2D {
        // 3x4 grid: lat from -10 to 10 (step 10), lon from 0 to 30 (step 10)
        //
        //           lon: 0   10   20   30
        // lat  10:       9   10   11   12
        // lat   0:       5    6    7    8
        // lat -10:       1    2    3    4
        //
        // Stored south-to-north (row 0 = lat -10):
        let data = vec![
            1.0, 2.0, 3.0, 4.0, // lat = -10
            5.0, 6.0, 7.0, 8.0, // lat =   0
            9.0, 10.0, 11.0, 12.0, // lat =  10
        ];
        RegularGrid2D::new(-10.0, 10.0, 3, 0.0, 10.0, 4, data)
    }

    #[test]
    fn test_grid_node_values() {
        let g = sample_grid();
        // Exact grid points should return exact values
        assert_approx_eq!(g.bilinear(-10.0, 0.0), 1.0, atol <= 1e-12);
        assert_approx_eq!(g.bilinear(0.0, 10.0), 6.0, atol <= 1e-12);
        assert_approx_eq!(g.bilinear(10.0, 30.0), 12.0, atol <= 1e-12);
    }

    #[test]
    fn test_bilinear_midpoint() {
        let g = sample_grid();
        // Midpoint between (lat=0, lon=0)=5 and (lat=0, lon=10)=6
        assert_approx_eq!(g.bilinear(0.0, 5.0), 5.5, atol <= 1e-12);
        // Midpoint between (lat=-10, lon=0)=1 and (lat=0, lon=0)=5
        assert_approx_eq!(g.bilinear(-5.0, 0.0), 3.0, atol <= 1e-12);
        // Centre of 4 cells: (1+2+5+6)/4 = 3.5
        assert_approx_eq!(g.bilinear(-5.0, 5.0), 3.5, atol <= 1e-12);
    }

    #[test]
    fn test_nearest() {
        let g = sample_grid();
        assert_approx_eq!(g.nearest(-10.0, 0.0), 1.0, atol <= 1e-12);
        assert_approx_eq!(g.nearest(-6.0, 4.0), 1.0, atol <= 1e-12);
        assert_approx_eq!(g.nearest(-4.0, 6.0), 6.0, atol <= 1e-12);
    }

    #[test]
    fn test_latitude_clamping() {
        let g = sample_grid();
        // Beyond north edge — clamp to lat=10
        assert_approx_eq!(g.bilinear(20.0, 0.0), 9.0, atol <= 1e-12);
        // Beyond south edge — clamp to lat=-10
        assert_approx_eq!(g.bilinear(-20.0, 10.0), 2.0, atol <= 1e-12);
    }

    #[test]
    fn test_longitude_wrapping() {
        let g = sample_grid();
        // lon=40 should wrap to lon=0 (grid span is 40: 4 columns * step 10)
        assert_approx_eq!(g.bilinear(0.0, 40.0), g.bilinear(0.0, 0.0), atol <= 1e-12);
        // lon=-10 should wrap to lon=30
        assert_approx_eq!(g.bilinear(0.0, -10.0), g.bilinear(0.0, 30.0), atol <= 1e-12);
    }
}
