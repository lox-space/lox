// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Module xy06 provides a function to calculate the (X, Y) position of the Celestial Intermediate
//! Pole (CIP) using the IAU 2006 precession and IAU 2000A nutation models.

use glam::DVec2;

use lox_core::types::units::JulianCenturies;
use lox_core::units::{Angle, AngleUnits};

use crate::iers::fundamental::iers03::{
    d_iers03, earth_l_iers03, f_iers03, jupiter_l_iers03, l_iers03, lp_iers03, mars_l_iers03,
    mercury_l_iers03, neptune_l_iers03, omega_iers03, pa_iers03, saturn_l_iers03, uranus_l_iers03,
    venus_l_iers03,
};

mod amplitudes;
mod luni_solar;
mod planetary;
mod polynomial;

const MAX_POWER_OF_T: usize = 5;

type PowersOfT = [f64; MAX_POWER_OF_T + 1];

type FundamentalArgs = [Angle; 14];

#[derive(Debug, Default)]
struct NutationComponents {
    planetary: DVec2,
    luni_solar: DVec2,
}

pub fn cip_coords(centuries_since_j2000_tdb: JulianCenturies) -> (Angle, Angle) {
    let powers_of_t = powers_of_t(centuries_since_j2000_tdb);
    let fundamental_args = fundamental_args(centuries_since_j2000_tdb);
    let polynomial_components = polynomial_components(&powers_of_t);
    let nutation_components = nutation_components(&powers_of_t, &fundamental_args);
    calculate_cip_unit_vector(&polynomial_components, &nutation_components)
}

fn powers_of_t(centuries_since_j2000_tdb: JulianCenturies) -> PowersOfT {
    let mut tn: f64 = 1.0;
    let mut powers_of_t = PowersOfT::default();
    for pow in powers_of_t.iter_mut() {
        *pow = tn;
        tn *= centuries_since_j2000_tdb;
    }
    powers_of_t
}

fn fundamental_args(centuries_since_j2000_tdb: JulianCenturies) -> FundamentalArgs {
    // The output of the CIP calculation is dependent on the ordering of these arguments. DO NOT
    // EDIT.
    [
        l_iers03(centuries_since_j2000_tdb),
        lp_iers03(centuries_since_j2000_tdb),
        f_iers03(centuries_since_j2000_tdb),
        d_iers03(centuries_since_j2000_tdb),
        omega_iers03(centuries_since_j2000_tdb),
        mercury_l_iers03(centuries_since_j2000_tdb),
        venus_l_iers03(centuries_since_j2000_tdb),
        earth_l_iers03(centuries_since_j2000_tdb),
        mars_l_iers03(centuries_since_j2000_tdb),
        jupiter_l_iers03(centuries_since_j2000_tdb),
        saturn_l_iers03(centuries_since_j2000_tdb),
        uranus_l_iers03(centuries_since_j2000_tdb),
        neptune_l_iers03(centuries_since_j2000_tdb),
        pa_iers03(centuries_since_j2000_tdb),
    ]
}

fn polynomial_components(powers_of_t: &PowersOfT) -> DVec2 {
    let mut result = DVec2::default();
    for (i, power_of_t) in powers_of_t.iter().enumerate().rev() {
        result[0] += polynomial::COEFFICIENTS.x[i] * power_of_t;
        result[1] += polynomial::COEFFICIENTS.y[i] * power_of_t;
    }
    result
}

fn nutation_components(
    powers_of_t: &PowersOfT,
    fundamental_args: &FundamentalArgs,
) -> NutationComponents {
    let mut result = NutationComponents::default();

    // The sin and cosine of the current argument, preallocated and dynamically accessible
    // (as opposed to a tuple).
    let mut sin_cos = [0.0; 2];

    // The last amplitude chunk to be processed.
    let mut last_amplitude_chunk_index = amplitudes::COEFFICIENTS.len();

    // Calculate planetary nutation components.
    for (freq_list_idx, freq_list) in planetary::FREQUENCY_LISTS.iter().enumerate().rev() {
        // Calculate argument functions.
        let mut arg = 0.0.rad();
        for (i, &freq) in freq_list.iter().enumerate() {
            arg += freq * fundamental_args[i];
        }
        sin_cos[0] = arg.sin();
        sin_cos[1] = arg.cos();

        // The list of indices into the amplitudes array contains both luni-solar and planetary
        // indices. We offset by the number of luni-solar frequency lists to get the correct
        // planetary index.
        let amplitude_indices_idx = freq_list_idx + luni_solar::N_FREQUENCY_LISTS;
        let current_amplitude_chunk_idx = amplitudes::INDICES[amplitude_indices_idx];

        // Iterate backwards through the amplitudes of the current frequency chunk.
        for i in (current_amplitude_chunk_idx..=last_amplitude_chunk_index).rev() {
            // The index of the current amplitude within the chunk.
            let relative_amplitude_idx = i - current_amplitude_chunk_idx;
            let axis = amplitudes::USAGE_XY[relative_amplitude_idx];
            let trig_func = amplitudes::USAGE_SIN_COS[relative_amplitude_idx];
            let power_of_t = amplitudes::USAGE_POWER_OF_T[relative_amplitude_idx];

            // Accumulate the component.
            result.planetary[axis] +=
                amplitudes::COEFFICIENTS[i - 1] * sin_cos[trig_func] * powers_of_t[power_of_t];
        }
        last_amplitude_chunk_index = current_amplitude_chunk_idx - 1;
    }

    // Calculate luni-solar nutation components.
    for (freq_list_idx, freq_list) in luni_solar::FREQUENCY_LISTS.iter().enumerate().rev() {
        // Calculate argument functions.
        let mut arg = 0.0.rad();
        for (i, &freq) in freq_list.iter().enumerate() {
            arg += freq * fundamental_args[i];
        }
        sin_cos[0] = arg.sin();
        sin_cos[1] = arg.cos();

        // The list of indices into the amplitudes array contains both luni-solar and planetary
        // indices. We offset by the number of luni-solar frequency lists to get the correct
        // luni-solar index.
        let amplitude_indices_idx = freq_list_idx;
        let current_amplitude_chunk_idx = amplitudes::INDICES[amplitude_indices_idx];

        // Iterate backwards through the amplitudes of the current frequency chunk.
        for i in (current_amplitude_chunk_idx..=last_amplitude_chunk_index).rev() {
            // The index of the current amplitude within the chunk.
            let relative_amplitude_idx = i - current_amplitude_chunk_idx;
            let axis = amplitudes::USAGE_XY[relative_amplitude_idx];
            let trig_func = amplitudes::USAGE_SIN_COS[relative_amplitude_idx];
            let power_of_t = amplitudes::USAGE_POWER_OF_T[relative_amplitude_idx];

            // Accumulate the component.
            result.luni_solar[axis] +=
                amplitudes::COEFFICIENTS[i - 1] * sin_cos[trig_func] * powers_of_t[power_of_t];
        }
        last_amplitude_chunk_index = current_amplitude_chunk_idx - 1;
    }

    result
}

fn calculate_cip_unit_vector(
    polynomial_components: &DVec2,
    nutation_components: &NutationComponents,
) -> (Angle, Angle) {
    let x = polynomial_components[0]
        + (nutation_components.planetary[0] + nutation_components.luni_solar[0]) / 1e6;
    let y = polynomial_components[1]
        + (nutation_components.planetary[1] + nutation_components.luni_solar[1]) / 1e6;
    (x.arcsec(), y.arcsec())
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;
    use lox_time::{Time, time_scales::Tdb};

    use crate::iers::cip::CipCoords;

    use super::*;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_cip_coordinates_iau2006() {
        let time = Time::from_two_part_julian_date(Tdb, 2400000.5, 53736.0);
        let CipCoords { x, y } = CipCoords::iau2006(time);
        assert_approx_eq!(x, 5.791_308_486_706_011e-4.rad(), rtol <= TOLERANCE);
        assert_approx_eq!(y, 4.020_579_816_732_958e-5.rad(), rtol <= TOLERANCE);
    }

    #[test]
    fn test_fundamental_args_ordering() {
        let j2000: JulianCenturies = 0.0;
        let act = fundamental_args(j2000);
        let exp = [
            l_iers03(j2000),
            lp_iers03(j2000),
            f_iers03(j2000),
            d_iers03(j2000),
            omega_iers03(j2000),
            mercury_l_iers03(j2000),
            venus_l_iers03(j2000),
            earth_l_iers03(j2000),
            mars_l_iers03(j2000),
            jupiter_l_iers03(j2000),
            saturn_l_iers03(j2000),
            uranus_l_iers03(j2000),
            neptune_l_iers03(j2000),
            pa_iers03(j2000),
        ];

        assert_approx_eq!(act, exp, rtol <= TOLERANCE)
    }
}
