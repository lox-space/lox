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
use crate::time::intervals::TDBJulianCenturiesSinceJ2000;
use crate::types::Radians;

type XY = [f64; 2];

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
fn cip_xy(t: TDBJulianCenturiesSinceJ2000) -> XY {
    let powers_of_t = powers_of_t(t);
    let fundamental_args = fundamental_args(t);
    let polynomial_components = polynomial_components(&powers_of_t);
    let planetary_nutation_components = nutation_components(&powers_of_t, &fundamental_args);
    let mut luni_solar_xy_totals = (0.0, 0.0);
    let mut planetary_xy_totals = (0.0, 0.0);
    [0.0, 0.0]
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

    // The last amplitude to be processed.
    let mut last_amplitudes_index = amplitudes::COEFFICIENTS.len();

    for (freq_list_idx, freq_list) in planetary::FREQUENCY_LISTS.iter().enumerate().rev() {
        // Calculate argument functions.
        let mut arg = 0.0;
        for (i, freq) in freq_list.iter().enumerate() {
            arg += freq * fundamental_args[i];
        }
        sin_cos[0] = arg.sin();
        sin_cos[1] = arg.cos();

        // Iterate backwards through the amplitudes at the current frequency.

        // The list of indices into the amplitudes array contains both luni-solar and planetary
        // indices. We offset by the number of luni-solar frequency lists to get the correct
        // planetary index.
        let amplitude_indices_idx = freq_list_idx + luni_solar::N_FREQUENCY_LISTS;
        let amplitude_idx = amplitudes::INDICES[amplitude_indices_idx];
        amplitudes::COEFFICIENTS[amplitude_idx..last_amplitudes_index] // last_ampl_index is 1 greater than C but exclusive
            .iter()
            .enumerate()
            .rev()
            .for_each(|(i, amplitude)| {
                let axis = amplitudes::USAGE_XY[i];
                let trig_func = amplitudes::USAGE_SIN_COS[i];
                let power_of_t = amplitudes::USAGE_POWER_OF_T[i];

                // Accumulate the component.
                result.planetary[axis] += amplitude * sin_cos[trig_func] * powers_of_t[power_of_t];
            });
        last_amplitudes_index = amplitude_idx;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn test_nutation_components() {
        let t = 0.0;
        let powers_of_t = powers_of_t(t);
        let fundamental_args = fundamental_args(t);
        let result = nutation_components(&powers_of_t, &fundamental_args);
        assert_float_eq!(result.planetary[0], -8.595532, rel <= 1e-11);
        assert_float_eq!(result.planetary[1], 274.365087, rel <= 1e-11);
    }
}
