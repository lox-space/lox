/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod amplitudes;
mod luni_solar;
mod planetary;
mod polynomial;

use crate::bodies::fundamental::iers03::{
    general_accum_precession_in_longitude_iers03, mean_moon_sun_elongation_iers03,
};
use crate::bodies::{Earth, Jupiter, Mars, Mercury, Moon, Neptune, Saturn, Sun, Uranus, Venus};
use crate::math::arcsec_to_rad;
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;
use crate::types::Radians;

/// A convenient type for performing batch mathematical operations on X and Y components. This
/// type may change or become unexported as the needs of upstream components become clearer.
pub type XY = [f64; 2];

const MAX_POWER_OF_T: usize = 5;

type PowersOfT = [f64; MAX_POWER_OF_T + 1];

type FundamentalArgs = [Radians; 14];

struct PolynomialCoefficients {
    x: [f64; MAX_POWER_OF_T + 1],
    y: [f64; MAX_POWER_OF_T + 1],
}

type LuniSolarFrequencyList = [f64; 5];

type PlanetaryFrequencyList = [f64; 14];

type MicroArcsecond = f64;

#[derive(Default)]
struct NutationComponents {
    planetary: XY,
    luni_solar: XY,
}

/// (X, Y) coordinates of the Celestial Intermediate Pole (CIP) using the the IAU 2006 precession
/// and IAU 2000A nutation models.
pub fn cip_xy(t: TDBJulianCenturiesSinceJ2000) -> XY {
    let powers_of_t = powers_of_t(t);
    let fundamental_args = fundamental_args(t);
    let polynomial_components = polynomial_components(&powers_of_t);
    let nutation_components = nutation_components(&powers_of_t, &fundamental_args);
    calculate_cip_unit_vector(&polynomial_components, &nutation_components)
}

fn powers_of_t(t: TDBJulianCenturiesSinceJ2000) -> PowersOfT {
    let mut tn: f64 = 1.0;
    let mut powers_of_t = [0.0; MAX_POWER_OF_T + 1];
    for pow in powers_of_t.iter_mut() {
        *pow = tn;
        tn *= t;
    }
    powers_of_t
}

/// The output of the CIP calculation is dependent on the ordering of these arguments. DO NOT EDIT.
fn fundamental_args(t: TDBJulianCenturiesSinceJ2000) -> FundamentalArgs {
    [
        Moon.mean_anomaly_iers03(t),
        Sun.mean_anomaly_iers03(t),
        Moon.mean_longitude_minus_ascending_node_mean_longitude_iers03(t),
        mean_moon_sun_elongation_iers03(t),
        Moon.ascending_node_mean_longitude_iers03(t),
        Mercury.mean_longitude_iers03(t),
        Venus.mean_longitude_iers03(t),
        Earth.mean_longitude_iers03(t),
        Mars.mean_longitude_iers03(t),
        Jupiter.mean_longitude_iers03(t),
        Saturn.mean_longitude_iers03(t),
        Uranus.mean_longitude_iers03(t),
        Neptune.mean_longitude_iers03(t),
        general_accum_precession_in_longitude_iers03(t),
    ]
}

fn polynomial_components(powers_of_t: &PowersOfT) -> XY {
    let mut result = [0.0; 2];
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
    polynomial_components: &XY,
    nutation_components: &NutationComponents,
) -> XY {
    let x_arcsec = polynomial_components[0]
        + (nutation_components.planetary[0] + nutation_components.luni_solar[0]) / 1e6;
    let y_arcsec = polynomial_components[1]
        + (nutation_components.planetary[1] + nutation_components.luni_solar[1]) / 1e6;
    let x = arcsec_to_rad(x_arcsec);
    let y = arcsec_to_rad(y_arcsec);
    [x, y]
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_cip_xy_jd0() {
        let jd0: TDBJulianCenturiesSinceJ2000 = -67.11964407939767;
        let xy = cip_xy(jd0);
        assert_float_eq!(xy[0], -0.4088355637476968, rel <= TOLERANCE);
        assert_float_eq!(xy[1], -0.38359667445777073, rel <= TOLERANCE);
    }

    #[test]
    fn test_cip_xy_j2000() {
        let j2000: TDBJulianCenturiesSinceJ2000 = 0.0;
        let xy = cip_xy(j2000);
        assert_float_eq!(xy[0], -0.0000269463795685740, rel <= TOLERANCE);
        assert_float_eq!(xy[1], -0.00002800472282281282, rel <= TOLERANCE);
    }

    #[test]
    fn test_cip_xy_j2100() {
        let j2100: TDBJulianCenturiesSinceJ2000 = 1.0;
        let xy = cip_xy(j2100);
        assert_float_eq!(xy[0], 0.00972070446172924, rel <= TOLERANCE);
        assert_float_eq!(xy[1], -0.0000673058699616719, rel <= TOLERANCE);
    }
}
