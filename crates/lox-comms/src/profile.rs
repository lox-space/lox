// SPDX-FileCopyrightText: 2026 Torkil Rein Gustavsen <torkilrg@ksat.no>
//
// SPDX-License-Identifier: MPL-2.0

//! Elevation-dependent propagation losses of a ground site.
//!
//! Atmospheric loss on an Earth–space path grows steeply towards the horizon:
//! at 5° elevation it is roughly five times the loss at 30°. An
//! [`ElevationProfile`] tabulates [`AtmosphericLosses`] over a grid of
//! elevations for one carrier and interpolates between them, so a link budget
//! can follow the loss along a pass instead of applying one fixed figure.
//! [`SitePropagation`] collects the profiles of one site, one per carrier.

use lox_core::units::{Angle, AngleUnit, Frequency, FrequencyUnit};

use crate::losses::AtmosphericLosses;

/// Relative carrier mismatch up to which a profile is considered to describe
/// a carrier (1 %).
///
/// Atmospheric loss varies slowly with frequency, so a profile computed at
/// 26.0 GHz remains representative at 26.25 GHz. Beyond the tolerance the
/// profile is not applied.
pub const CARRIER_MATCH_TOLERANCE: f64 = 0.01;

/// Errors produced while constructing an [`ElevationProfile`].
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ElevationProfileError {
    /// The profile has no samples.
    #[error("an elevation profile requires at least one sample")]
    Empty,
    /// A sample elevation lies outside `(0°, 90°]`.
    #[error(
        "sample elevations must lie in (0°, 90°], got {}",
        .elevation.display_in(AngleUnit::Degrees)
    )]
    ElevationOutOfRange {
        /// Offending elevation.
        elevation: Angle,
    },
    /// The sample elevations are not strictly increasing.
    #[error(
        "sample elevations must be strictly increasing, but {} is not",
        .elevation.display_in(AngleUnit::Degrees)
    )]
    NotIncreasing {
        /// First elevation that does not exceed its predecessor.
        elevation: Angle,
    },
    /// The carrier frequency is not finite and positive.
    #[error("the carrier must be finite and positive, got {carrier}")]
    InvalidCarrier {
        /// Offending carrier.
        carrier: Frequency,
    },
}

/// Propagation losses of one site at one carrier, as a function of elevation.
///
/// Holds [`AtmosphericLosses`] at a strictly increasing grid of elevations
/// and interpolates each of the four losses separately, so the result keeps
/// its itemization (and with it the absorptive share that degrades the
/// receive noise temperature).
///
/// Interpolation is linear in the cosecant of the elevation — the slant path
/// length through a plane-parallel atmosphere — which reproduces gaseous and
/// cloud attenuation almost exactly and tracks rain and scintillation closely
/// on a moderately dense grid. Outside the grid the profile holds its end
/// values: no extrapolation towards the horizon, where the underlying models
/// stop being valid anyway.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ElevationProfileRepr")
)]
pub struct ElevationProfile {
    carrier: Frequency,
    samples: Vec<(Angle, AtmosphericLosses)>,
    /// `1 / sin(elevation)` of each sample — the abscissa the profile is
    /// interpolated in. Derived from `samples` once, at construction.
    #[cfg_attr(feature = "serde", serde(skip))]
    cosecants: Vec<f64>,
}

/// Returns the cosecant of an elevation in radians.
fn cosecant(elevation: f64) -> f64 {
    1.0 / elevation.sin()
}

/// Serde wire format for [`ElevationProfile`]: forces deserialization through
/// the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ElevationProfileRepr {
    carrier: Frequency,
    samples: Vec<(Angle, AtmosphericLosses)>,
}

#[cfg(feature = "serde")]
impl TryFrom<ElevationProfileRepr> for ElevationProfile {
    type Error = ElevationProfileError;

    fn try_from(repr: ElevationProfileRepr) -> Result<Self, Self::Error> {
        ElevationProfile::new(repr.carrier, repr.samples)
    }
}

impl ElevationProfile {
    /// Creates a profile for `carrier` from `(elevation, losses)` samples.
    ///
    /// The elevations must be strictly increasing and lie in `(0°, 90°]`. A
    /// single sample yields an elevation-independent profile.
    pub fn new(
        carrier: Frequency,
        samples: Vec<(Angle, AtmosphericLosses)>,
    ) -> Result<Self, ElevationProfileError> {
        let carrier_hz = carrier.to_hertz();
        if !carrier_hz.is_finite() || carrier_hz <= 0.0 {
            return Err(ElevationProfileError::InvalidCarrier { carrier });
        }
        if samples.is_empty() {
            return Err(ElevationProfileError::Empty);
        }
        let mut previous = f64::NEG_INFINITY;
        for &(elevation, _) in &samples {
            let elevation_deg = elevation.to_degrees();
            // `!(..)` so that NaN is rejected too.
            if !(elevation_deg > 0.0 && elevation_deg <= 90.0) {
                return Err(ElevationProfileError::ElevationOutOfRange { elevation });
            }
            if elevation_deg <= previous {
                return Err(ElevationProfileError::NotIncreasing { elevation });
            }
            previous = elevation_deg;
        }
        let cosecants = samples
            .iter()
            .map(|(elevation, _)| cosecant(elevation.to_radians()))
            .collect();
        Ok(Self {
            carrier,
            samples,
            cosecants,
        })
    }

    /// Returns the carrier the profile was computed for.
    pub fn carrier(&self) -> Frequency {
        self.carrier
    }

    /// Returns the `(elevation, losses)` samples in increasing elevation.
    pub fn samples(&self) -> &[(Angle, AtmosphericLosses)] {
        &self.samples
    }

    /// Returns whether the profile describes `carrier`, i.e. whether the two
    /// agree within [`CARRIER_MATCH_TOLERANCE`].
    pub fn matches(&self, carrier: Frequency) -> bool {
        self.mismatch(carrier) <= CARRIER_MATCH_TOLERANCE
    }

    /// Returns the relative distance between `carrier` and the profile's own.
    fn mismatch(&self, carrier: Frequency) -> f64 {
        let own = self.carrier.to_hertz();
        ((carrier.to_hertz() - own) / own).abs()
    }

    /// Returns the losses at the given elevation.
    ///
    /// Interpolates linearly in `1 / sin(elevation)` between the two
    /// neighbouring samples and holds the end values outside the grid. An
    /// undefined (NaN) elevation yields the lowest sample, the most
    /// conservative one, so the result is always a valid set of losses.
    pub fn at(&self, elevation: Angle) -> AtmosphericLosses {
        let el = elevation.to_radians();
        // No sample compares `<=` to NaN, which therefore lands below the
        // grid.
        let upper = self.samples.partition_point(|(e, _)| e.to_radians() <= el);
        if upper == 0 {
            return self.samples[0].1;
        }
        if upper == self.samples.len() {
            return self.samples[upper - 1].1;
        }
        let (lo, hi) = (&self.samples[upper - 1].1, &self.samples[upper].1);
        let (x_lo, x_hi) = (self.cosecants[upper - 1], self.cosecants[upper]);
        let span = x_hi - x_lo;
        // Close to the zenith the cosecant is flat enough for two distinct
        // elevations to share one: there is nothing to interpolate over.
        if span == 0.0 {
            return *lo;
        }
        let t = ((cosecant(el) - x_lo) / span).clamp(0.0, 1.0);
        lo.lerp(hi, t)
    }
}

/// Errors produced while constructing a [`SitePropagation`].
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum SitePropagationError {
    /// Two profiles describe carriers too close to tell apart.
    #[error(
        "profiles at {} and {} overlap; \
         carriers must differ by more than {tolerance_percent} %",
        .first.display_in(FrequencyUnit::Gigahertz),
        .second.display_in(FrequencyUnit::Gigahertz)
    )]
    OverlappingCarriers {
        /// Carrier of the first profile.
        first: Frequency,
        /// Carrier of the second profile.
        second: Frequency,
        /// The matching tolerance in percent.
        tolerance_percent: f64,
    },
}

/// The elevation-dependent propagation losses of one ground site: one
/// [`ElevationProfile`] per carrier.
///
/// A site typically carries two profiles, one for the downlink carrier and
/// one for the uplink carrier. A link looks its profile up by carrier via
/// [`Self::for_carrier`], so the direction never has to be spelled out.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "SitePropagationRepr")
)]
pub struct SitePropagation {
    profiles: Vec<ElevationProfile>,
}

/// Serde wire format for [`SitePropagation`]: forces deserialization through
/// the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct SitePropagationRepr {
    profiles: Vec<ElevationProfile>,
}

#[cfg(feature = "serde")]
impl TryFrom<SitePropagationRepr> for SitePropagation {
    type Error = SitePropagationError;

    fn try_from(repr: SitePropagationRepr) -> Result<Self, Self::Error> {
        SitePropagation::new(repr.profiles)
    }
}

impl SitePropagation {
    /// Creates the propagation description of a site from its profiles.
    ///
    /// Returns [`SitePropagationError::OverlappingCarriers`] when the
    /// carriers of two profiles lie within [`CARRIER_MATCH_TOLERANCE`] of
    /// each other. A carrier between two profiles just over the tolerance
    /// apart can still match both; [`Self::for_carrier`] then picks the
    /// nearer one.
    pub fn new(profiles: Vec<ElevationProfile>) -> Result<Self, SitePropagationError> {
        for (i, first) in profiles.iter().enumerate() {
            for second in &profiles[i + 1..] {
                if first.matches(second.carrier) || second.matches(first.carrier) {
                    return Err(SitePropagationError::OverlappingCarriers {
                        first: first.carrier,
                        second: second.carrier,
                        tolerance_percent: CARRIER_MATCH_TOLERANCE * 100.0,
                    });
                }
            }
        }
        Ok(Self { profiles })
    }

    /// Returns the profiles of this site.
    pub fn profiles(&self) -> &[ElevationProfile] {
        &self.profiles
    }

    /// Returns the profile describing `carrier`, or `None` when no profile
    /// lies within [`CARRIER_MATCH_TOLERANCE`] of it.
    pub fn for_carrier(&self, carrier: Frequency) -> Option<&ElevationProfile> {
        self.profiles
            .iter()
            .filter(|profile| profile.matches(carrier))
            .min_by(|a, b| a.mismatch(carrier).total_cmp(&b.mismatch(carrier)))
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_core::units::Decibel;

    use super::*;

    fn deg(value: f64) -> Angle {
        Angle::degrees(value)
    }

    fn ghz(value: f64) -> Frequency {
        Frequency::gigahertz(value)
    }

    /// Gaseous loss that follows the cosecant law exactly, plus a rain line
    /// that does not.
    fn losses(zenith_gas_db: f64, elevation_deg: f64, rain_db: f64) -> AtmosphericLosses {
        AtmosphericLosses::new(
            Decibel::decibels(rain_db),
            Decibel::decibels(zenith_gas_db / elevation_deg.to_radians().sin()),
            Decibel::decibels(0.0),
            Decibel::decibels(0.0),
        )
        .unwrap()
    }

    fn profile() -> ElevationProfile {
        ElevationProfile::new(
            ghz(26.0),
            vec![
                (deg(5.0), losses(0.2, 5.0, 4.0)),
                (deg(10.0), losses(0.2, 10.0, 2.0)),
                (deg(30.0), losses(0.2, 30.0, 1.0)),
                (deg(90.0), losses(0.2, 90.0, 0.5)),
            ],
        )
        .unwrap()
    }

    #[test]
    fn test_profile_is_exact_at_its_samples() {
        let profile = profile();
        for (elevation, expected) in profile.samples() {
            assert_eq!(&profile.at(*elevation), expected);
        }
    }

    #[test]
    fn test_profile_reproduces_a_cosecant_law_between_samples() {
        let profile = profile();
        for elevation_deg in [6.0, 7.5, 12.0, 20.0, 45.0, 70.0] {
            let gas = profile.at(deg(elevation_deg)).gaseous().to_decibels();
            let expected = 0.2 / elevation_deg.to_radians().sin();
            assert_approx_eq!(gas, expected, rtol <= 1e-12);
        }
    }

    #[test]
    fn test_profile_interpolates_each_loss_separately() {
        let at = profile().at(deg(20.0));
        let rain = at.rain().to_decibels();
        assert!(rain > 1.0 && rain < 2.0, "rain at 20°: {rain}");
        assert_approx_eq!(
            at.gaseous().to_decibels(),
            0.2 / 20.0_f64.to_radians().sin(),
            rtol <= 1e-12
        );
        assert_eq!(at.cloud().to_decibels(), 0.0);
        assert_eq!(at.scintillation().to_decibels(), 0.0);
    }

    #[test]
    fn test_profile_holds_its_end_values_outside_the_grid() {
        let profile = profile();
        assert_eq!(profile.at(deg(1.0)), profile.samples()[0].1);
        assert_eq!(profile.at(deg(-3.0)), profile.samples()[0].1);
        assert_eq!(profile.at(deg(90.0)), profile.samples()[3].1);
    }

    /// Two samples so close to the zenith that their cosecants coincide span
    /// no interval to interpolate over: the lower one holds.
    #[test]
    fn test_profile_stays_finite_between_samples_of_equal_cosecant() {
        let (lo, hi) = (losses(0.2, 90.0, 1.0), losses(0.2, 90.0, 0.5));
        let profile = ElevationProfile::new(
            ghz(26.0),
            vec![
                (deg(30.0), losses(0.2, 30.0, 2.0)),
                (deg(89.9999999), lo),
                (deg(90.0), hi),
            ],
        )
        .unwrap();
        assert_eq!(profile.at(deg(89.99999995)), lo);
        assert_eq!(profile.at(deg(90.0)), hi);
    }

    /// An undefined elevation gets the lowest sample — the highest loss —
    /// rather than NaN losses.
    #[test]
    fn test_profile_holds_its_lowest_sample_at_a_nan_elevation() {
        let profile = profile();
        assert_eq!(profile.at(Angle::radians(f64::NAN)), profile.samples()[0].1);
    }

    #[test]
    fn test_single_sample_profile_is_constant() {
        let only = losses(0.2, 30.0, 1.0);
        let profile = ElevationProfile::new(ghz(26.0), vec![(deg(30.0), only)]).unwrap();
        for elevation_deg in [5.0, 30.0, 80.0] {
            assert_eq!(profile.at(deg(elevation_deg)), only);
        }
    }

    #[test]
    fn test_profile_rejects_invalid_samples() {
        let sample = |el: f64| (deg(el), losses(0.2, 10.0, 1.0));
        assert_eq!(
            ElevationProfile::new(ghz(26.0), vec![]),
            Err(ElevationProfileError::Empty)
        );
        for elevation_deg in [0.0, -5.0, 90.5, f64::NAN] {
            assert!(matches!(
                ElevationProfile::new(ghz(26.0), vec![sample(elevation_deg)]),
                Err(ElevationProfileError::ElevationOutOfRange { .. })
            ));
        }
        for second in [10.0, 5.0] {
            assert!(matches!(
                ElevationProfile::new(ghz(26.0), vec![sample(10.0), sample(second)]),
                Err(ElevationProfileError::NotIncreasing { .. })
            ));
        }
        for carrier_hz in [0.0, -1.0, f64::NAN] {
            assert!(matches!(
                ElevationProfile::new(Frequency::hertz(carrier_hz), vec![sample(10.0)]),
                Err(ElevationProfileError::InvalidCarrier { .. })
            ));
        }
    }

    #[test]
    fn test_errors_name_the_offending_value_in_its_customary_unit() {
        let sample = |el: f64| (deg(el), losses(0.2, 10.0, 1.0));
        let message = |result: Result<ElevationProfile, ElevationProfileError>| {
            result.unwrap_err().to_string()
        };
        assert_eq!(
            message(ElevationProfile::new(ghz(26.0), vec![sample(-5.0)])),
            "sample elevations must lie in (0°, 90°], got -5 deg"
        );
        assert_eq!(
            message(ElevationProfile::new(
                ghz(26.0),
                vec![sample(45.0), sample(45.0)]
            )),
            "sample elevations must be strictly increasing, but 45 deg is not"
        );
        assert_eq!(
            message(ElevationProfile::new(
                Frequency::hertz(-1.0),
                vec![sample(10.0)]
            )),
            "the carrier must be finite and positive, got -1 Hz"
        );
        assert_eq!(
            SitePropagation::new(vec![profile_at(26.0), profile_at(26.125)])
                .unwrap_err()
                .to_string(),
            "profiles at 26 GHz and 26.125 GHz overlap; carriers must differ by more than 1 %"
        );
    }

    fn profile_at(carrier_ghz: f64) -> ElevationProfile {
        ElevationProfile::new(ghz(carrier_ghz), vec![(deg(10.0), losses(0.2, 10.0, 1.0))]).unwrap()
    }

    #[test]
    fn test_site_propagation_looks_profiles_up_by_carrier() {
        let site = SitePropagation::new(vec![profile_at(26.0), profile_at(8.0)]).unwrap();
        assert_eq!(site.for_carrier(ghz(26.0)).unwrap().carrier(), ghz(26.0));
        // Within the 1 % tolerance.
        assert_eq!(site.for_carrier(ghz(26.25)).unwrap().carrier(), ghz(26.0));
        assert_eq!(site.for_carrier(ghz(7.95)).unwrap().carrier(), ghz(8.0));
        // Outside it: no profile, the caller falls back.
        assert!(site.for_carrier(ghz(26.5)).is_none());
        assert!(site.for_carrier(ghz(2.2)).is_none());
        assert!(SitePropagation::default().for_carrier(ghz(26.0)).is_none());
    }

    #[test]
    fn test_site_propagation_rejects_overlapping_carriers() {
        assert!(matches!(
            SitePropagation::new(vec![profile_at(26.0), profile_at(26.1)]),
            Err(SitePropagationError::OverlappingCarriers { .. })
        ));
        assert!(SitePropagation::new(vec![profile_at(26.0), profile_at(27.0)]).is_ok());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_site_propagation_serde_round_trip_and_validation() {
        let site = SitePropagation::new(vec![profile(), profile_at(8.0)]).unwrap();
        let json = serde_json::to_string(&site).unwrap();
        let round_trip: SitePropagation = serde_json::from_str(&json).unwrap();
        assert_eq!(site, round_trip);

        // A sample at 0° elevation must be rejected at deserialization time.
        let zero = r#"{"rain":0.0,"gaseous":0.0,"cloud":0.0,"scintillation":0.0}"#;
        let valid = json.replacen("\"samples\":[", &format!("\"samples\":[[0.01,{zero}],"), 1);
        assert!(serde_json::from_str::<SitePropagation>(&valid).is_ok());
        let bad = json.replacen("\"samples\":[", &format!("\"samples\":[[0.0,{zero}],"), 1);
        assert!(serde_json::from_str::<SitePropagation>(&bad).is_err());
    }
}
