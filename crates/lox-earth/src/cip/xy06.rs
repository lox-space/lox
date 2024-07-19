/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Module xy06 provides a function to calculate the (X, Y) position of the Celestial Intermediate
//! Pole (CIP) using the IAU 2006 precession and IAU 2000A nutation models.

use glam::DVec2;

use lox_bodies::fundamental::iers03::{
    general_accum_precession_in_longitude_iers03, mean_moon_sun_elongation_iers03,
};
use lox_bodies::{Earth, Jupiter, Mars, Mercury, Moon, Neptune, Saturn, Sun, Uranus, Venus};
use lox_math::math::arcsec_to_rad;
use lox_math::types::units::{JulianCenturies, Radians};

mod amplitudes;
mod luni_solar;
mod planetary;
mod polynomial;

const MAX_POWER_OF_T: usize = 5;

type PowersOfT = [f64; MAX_POWER_OF_T + 1];

type FundamentalArgs = [Radians; 14];

#[derive(Debug, Default)]
struct NutationComponents {
    planetary: DVec2,
    luni_solar: DVec2,
}

/// Calculates the (X, Y) coordinates of the Celestial Intermediate Pole (CIP) using the the IAU
/// 2006 precession and IAU 2000A nutation models.
pub fn xy(centuries_since_j2000_tdb: JulianCenturies) -> DVec2 {
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
        Moon.mean_anomaly_iers03(centuries_since_j2000_tdb),
        Sun.mean_anomaly_iers03(centuries_since_j2000_tdb),
        Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(centuries_since_j2000_tdb),
        mean_moon_sun_elongation_iers03(centuries_since_j2000_tdb),
        Moon.ascending_node_mean_longitude_iers03(centuries_since_j2000_tdb),
        Mercury.mean_longitude_iers03(centuries_since_j2000_tdb),
        Venus.mean_longitude_iers03(centuries_since_j2000_tdb),
        Earth.mean_longitude_iers03(centuries_since_j2000_tdb),
        Mars.mean_longitude_iers03(centuries_since_j2000_tdb),
        Jupiter.mean_longitude_iers03(centuries_since_j2000_tdb),
        Saturn.mean_longitude_iers03(centuries_since_j2000_tdb),
        Uranus.mean_longitude_iers03(centuries_since_j2000_tdb),
        Neptune.mean_longitude_iers03(centuries_since_j2000_tdb),
        general_accum_precession_in_longitude_iers03(centuries_since_j2000_tdb),
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
        let mut arg = 0.0;
        for (i, freq) in freq_list.iter().enumerate() {
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
        let mut arg = 0.0;
        for (i, freq) in freq_list.iter().enumerate() {
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
) -> DVec2 {
    let x_arcsec = polynomial_components[0]
        + (nutation_components.planetary[0] + nutation_components.luni_solar[0]) / 1e6;
    let y_arcsec = polynomial_components[1]
        + (nutation_components.planetary[1] + nutation_components.luni_solar[1]) / 1e6;
    DVec2 {
        x: arcsec_to_rad(x_arcsec),
        y: arcsec_to_rad(y_arcsec),
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_cip_xy_jd0() {
        let jd0: JulianCenturies = -67.11964407939767;
        let xy = xy(jd0);
        assert_float_eq!(xy[0], -0.4088355637476968, rel <= TOLERANCE);
        assert_float_eq!(xy[1], -0.38359667445777073, rel <= TOLERANCE);
    }

    #[test]
    fn test_cip_xy_j2000() {
        let j2000: JulianCenturies = 0.0;
        let xy = xy(j2000);
        assert_float_eq!(xy[0], -0.0000269463795685740, rel <= TOLERANCE);
        assert_float_eq!(xy[1], -0.00002800472282281282, rel <= TOLERANCE);
    }

    #[test]
    fn test_cip_xy_j2100() {
        let j2100: JulianCenturies = 1.0;
        let xy = xy(j2100);
        assert_float_eq!(xy[0], 0.00972070446172924, rel <= TOLERANCE);
        assert_float_eq!(xy[1], -0.0000673058699616719, rel <= TOLERANCE);
    }

    #[test]
    fn test_fundamental_args_ordering() {
        let j2000: JulianCenturies = 0.0;
        let actual = fundamental_args(j2000);
        let expected = [
            Moon.mean_anomaly_iers03(j2000),
            Sun.mean_anomaly_iers03(j2000),
            Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(j2000),
            mean_moon_sun_elongation_iers03(j2000),
            Moon.ascending_node_mean_longitude_iers03(j2000),
            Mercury.mean_longitude_iers03(j2000),
            Venus.mean_longitude_iers03(j2000),
            Earth.mean_longitude_iers03(j2000),
            Mars.mean_longitude_iers03(j2000),
            Jupiter.mean_longitude_iers03(j2000),
            Saturn.mean_longitude_iers03(j2000),
            Uranus.mean_longitude_iers03(j2000),
            Neptune.mean_longitude_iers03(j2000),
            general_accum_precession_in_longitude_iers03(j2000),
        ];

        expected.iter().enumerate().for_each(|(i, expected)| {
            assert_float_eq!(*expected, actual[i], rel <= TOLERANCE);
        });
    }
}
