// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Elevation-dependent propagation losses of a ground site.
//!
//! Atmospheric loss on an Earth–space path grows steeply towards the horizon:
//! at 5° elevation it is roughly five times the loss at 30°. An
//! [`ElevationProfile`] tabulates [`PropagationLosses`] over a grid of
//! elevations for one carrier and interpolates between them, so a link budget
//! can follow the loss along a pass instead of applying one fixed figure.
//! [`SitePropagation`] collects the profiles of one site, one per carrier.

use lox_core::units::{Angle, Frequency};

use crate::losses::PropagationLosses;

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
    #[error("sample elevations must lie in (0°, 90°], got {elevation_deg}°")]
    ElevationOutOfRange {
        /// Offending elevation in degrees.
        elevation_deg: f64,
    },
    /// The sample elevations are not strictly increasing.
    #[error("sample elevations must be strictly increasing, but {elevation_deg}° is not")]
    NotIncreasing {
        /// First elevation, in degrees, that does not exceed its predecessor.
        elevation_deg: f64,
    },
    /// A sample lists different loss kinds than the first one.
    #[error("the sample at {elevation_deg}° lists different loss kinds than the first sample")]
    MismatchedKinds {
        /// Elevation of the offending sample in degrees.
        elevation_deg: f64,
    },
    /// The carrier frequency is not finite and positive.
    #[error("the carrier must be finite and positive, got {carrier_hz} Hz")]
    InvalidCarrier {
        /// Offending carrier in Hz.
        carrier_hz: f64,
    },
}

/// Propagation losses of one site at one carrier, as a function of elevation.
///
/// Holds [`PropagationLosses`] at a strictly increasing grid of elevations.
/// Every sample lists the same loss kinds in the same order, so the profile
/// interpolates line by line and the result keeps its itemization (and with
/// it the absorptive share that degrades the receive noise temperature).
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
    samples: Vec<(Angle, PropagationLosses)>,
}

/// Serde wire format for [`ElevationProfile`]: forces deserialization through
/// the validated constructor.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ElevationProfileRepr {
    carrier: Frequency,
    samples: Vec<(Angle, PropagationLosses)>,
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
    /// The elevations must be strictly increasing and lie in `(0°, 90°]`, and
    /// every sample must list the same loss kinds in the same order. A single
    /// sample yields an elevation-independent profile.
    pub fn new(
        carrier: Frequency,
        samples: Vec<(Angle, PropagationLosses)>,
    ) -> Result<Self, ElevationProfileError> {
        let carrier_hz = carrier.to_hertz();
        if !carrier_hz.is_finite() || carrier_hz <= 0.0 {
            return Err(ElevationProfileError::InvalidCarrier { carrier_hz });
        }
        let Some((_, first)) = samples.first() else {
            return Err(ElevationProfileError::Empty);
        };
        let mut previous = f64::NEG_INFINITY;
        for (elevation, losses) in &samples {
            let elevation_deg = elevation.to_degrees();
            // `!(..)` so that NaN is rejected too.
            if !(elevation_deg > 0.0 && elevation_deg <= 90.0) {
                return Err(ElevationProfileError::ElevationOutOfRange { elevation_deg });
            }
            if elevation_deg <= previous {
                return Err(ElevationProfileError::NotIncreasing { elevation_deg });
            }
            if !first.same_kinds(losses) {
                return Err(ElevationProfileError::MismatchedKinds { elevation_deg });
            }
            previous = elevation_deg;
        }
        Ok(Self { carrier, samples })
    }

    /// Returns the carrier the profile was computed for.
    pub fn carrier(&self) -> Frequency {
        self.carrier
    }

    /// Returns the `(elevation, losses)` samples in increasing elevation.
    pub fn samples(&self) -> &[(Angle, PropagationLosses)] {
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
    /// neighbouring samples and holds the end values outside the grid.
    pub fn at(&self, elevation: Angle) -> PropagationLosses {
        let el = elevation.to_radians();
        let upper = self.samples.partition_point(|(e, _)| e.to_radians() <= el);
        if upper == 0 {
            return self.samples[0].1.clone();
        }
        if upper == self.samples.len() {
            return self.samples[upper - 1].1.clone();
        }
        let (el_lo, lo) = &self.samples[upper - 1];
        let (el_hi, hi) = &self.samples[upper];
        let csc = |angle: f64| 1.0 / angle.sin();
        let (x_lo, x_hi) = (csc(el_lo.to_radians()), csc(el_hi.to_radians()));
        let t = ((csc(el) - x_lo) / (x_hi - x_lo)).clamp(0.0, 1.0);
        lo.lerp(hi, t)
    }
}

/// Errors produced while constructing a [`SitePropagation`].
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum SitePropagationError {
    /// Two profiles describe carriers too close to tell apart.
    #[error(
        "profiles at {first_ghz} GHz and {second_ghz} GHz overlap; \
         carriers must differ by more than {tolerance_percent} %"
    )]
    OverlappingCarriers {
        /// Carrier of the first profile in GHz.
        first_ghz: f64,
        /// Carrier of the second profile in GHz.
        second_ghz: f64,
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
    /// Returns [`SitePropagationError::OverlappingCarriers`] when two
    /// profiles would both match the same carrier.
    pub fn new(profiles: Vec<ElevationProfile>) -> Result<Self, SitePropagationError> {
        for (i, first) in profiles.iter().enumerate() {
            for second in &profiles[i + 1..] {
                if first.matches(second.carrier) || second.matches(first.carrier) {
                    return Err(SitePropagationError::OverlappingCarriers {
                        first_ghz: first.carrier.to_gigahertz(),
                        second_ghz: second.carrier.to_gigahertz(),
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
    fn losses(zenith_gas_db: f64, elevation_deg: f64, rain_db: f64) -> PropagationLosses {
        PropagationLosses::builder()
            .gaseous(Decibel::decibels(
                zenith_gas_db / elevation_deg.to_radians().sin(),
            ))
            .rain(Decibel::decibels(rain_db))
            .build()
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
            let gas = profile.at(deg(elevation_deg)).lines()[0]
                .value()
                .to_decibels();
            let expected = 0.2 / elevation_deg.to_radians().sin();
            assert_approx_eq!(gas, expected, rtol <= 1e-12);
        }
    }

    #[test]
    fn test_profile_interpolates_every_line_and_keeps_the_itemization() {
        let at = profile().at(deg(20.0));
        let labels: Vec<_> = at.lines().iter().map(|l| l.kind().label()).collect();
        assert_eq!(labels, ["Gaseous absorption", "Rain attenuation"]);
        let rain = at.lines()[1].value().to_decibels();
        assert!(rain > 1.0 && rain < 2.0, "rain at 20°: {rain}");
        // Both lines are absorptive, so the absorptive share is the total.
        assert_approx_eq!(
            at.absorptive().to_decibels(),
            at.total().to_decibels(),
            rtol <= 1e-12
        );
    }

    #[test]
    fn test_profile_holds_its_end_values_outside_the_grid() {
        let profile = profile();
        assert_eq!(profile.at(deg(1.0)), profile.samples()[0].1);
        assert_eq!(profile.at(deg(-3.0)), profile.samples()[0].1);
        assert_eq!(profile.at(deg(90.0)), profile.samples()[3].1);
    }

    #[test]
    fn test_single_sample_profile_is_constant() {
        let only = losses(0.2, 30.0, 1.0);
        let profile = ElevationProfile::new(ghz(26.0), vec![(deg(30.0), only.clone())]).unwrap();
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
        let other_kinds = PropagationLosses::builder()
            .rain(Decibel::decibels(1.0))
            .build()
            .unwrap();
        assert!(matches!(
            ElevationProfile::new(ghz(26.0), vec![sample(10.0), (deg(20.0), other_kinds)]),
            Err(ElevationProfileError::MismatchedKinds { .. })
        ));
        for carrier_hz in [0.0, -1.0, f64::NAN] {
            assert!(matches!(
                ElevationProfile::new(Frequency::hertz(carrier_hz), vec![sample(10.0)]),
                Err(ElevationProfileError::InvalidCarrier { .. })
            ));
        }
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

        // An unsorted grid must be rejected at deserialization time.
        let unsorted = ElevationProfileRepr {
            carrier: ghz(26.0),
            samples: vec![],
        };
        assert!(ElevationProfile::try_from(unsorted).is_err());
        let bad = json.replacen("\"samples\":[", "\"samples\":[[0.0,{\"lines\":[]}],", 1);
        assert!(serde_json::from_str::<SitePropagation>(&bad).is_err());
    }
}
