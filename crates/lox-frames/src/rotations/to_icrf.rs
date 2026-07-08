// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Rotations between reference frames, composed through ICRF.
//!
//! Every frame implements [`RotateToIcrf`], giving its rotation to and from
//! ICRF. The blanket [`TryRotation`] impl uses these to rotate between any two
//! frames.
//!
//! [`TryRotation`]: crate::rotations::TryRotation

use lox_bodies::TryRotationalElements;
use lox_time::{
    Time,
    offsets::TryOffset,
    time_scales::{Tdb, TimeScale, Tt, Ut1},
};

use crate::{
    DynFrame,
    frames::{Cirf, Iau, Icrf, Itrf, J2000, Mod, Pef, Teme, Tirf, Tod},
    iers::{IersSystem, ReferenceSystem},
    rotations::{Rotation, RotationError, RotationProvider, TryRotation},
    traits::{ReferenceFrame, frame_id},
};

/// A frame that can produce its own rotation to and from ICRF from a provider's data.
pub trait RotateToIcrf<T: TimeScale, P> {
    /// The error type returned when the rotation cannot be computed.
    type Error;

    /// Returns the rotation from this frame to ICRF at `time`.
    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error>;

    /// Returns the rotation from ICRF to this frame at `time`.
    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error>;
}

/// Rotation from `origin` to `target`, composed through ICRF.
pub fn rotation_via_icrf<T, P, O, Tg>(
    provider: &P,
    origin: O,
    target: Tg,
    time: Time<T>,
) -> Result<Rotation, O::Error>
where
    T: TimeScale + Copy,
    O: RotateToIcrf<T, P>,
    Tg: RotateToIcrf<T, P, Error = O::Error>,
{
    let origin_to_icrf = origin.rotation_to_icrf(provider, time)?;
    let icrf_to_target = target.rotation_from_icrf(provider, time)?;
    Ok(origin_to_icrf.compose(icrf_to_target))
}

/// Blanket rotation between any two frames that know their route to ICRF.
impl<T, O, Tg, P> TryRotation<O, Tg, T> for P
where
    T: TimeScale + Copy,
    O: ReferenceFrame + RotateToIcrf<T, P, Error = RotationError>,
    Tg: ReferenceFrame + RotateToIcrf<T, P, Error = RotationError>,
{
    type Error = RotationError;

    fn try_rotation(&self, origin: O, target: Tg, time: Time<T>) -> Result<Rotation, Self::Error> {
        // Skip work cheaply via frame_id: identical frames need no rotation, and
        // when one endpoint is ICRF a single leg suffices (no composition).
        let origin_id = frame_id(&origin);
        let target_id = frame_id(&target);
        let icrf_id = frame_id(&Icrf);
        if origin_id.is_some() && origin_id == target_id {
            Ok(Rotation::IDENTITY)
        } else if origin_id == icrf_id {
            target.rotation_from_icrf(self, time)
        } else if target_id == icrf_id {
            origin.rotation_to_icrf(self, time)
        } else {
            rotation_via_icrf(self, origin, target, time)
        }
    }
}

// ---- the hub ---------------------------------------------------------------

impl<T, P> RotateToIcrf<T, P> for Icrf
where
    T: TimeScale + Copy,
    P: RotationProvider<T>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, _provider: &P, _time: Time<T>) -> Result<Rotation, Self::Error> {
        Ok(Rotation::IDENTITY)
    }

    fn rotation_from_icrf(&self, _provider: &P, _time: Time<T>) -> Result<Rotation, Self::Error> {
        Ok(Rotation::IDENTITY)
    }
}

// ---- quasi-inertial: frame bias / body-fixed -------------------------------

impl<T, P> RotateToIcrf<T, P> for J2000
where
    T: TimeScale + Copy,
    P: RotationProvider<T>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, _time: Time<T>) -> Result<Rotation, Self::Error> {
        Ok(provider.j2000_to_icrf())
    }

    fn rotation_from_icrf(&self, provider: &P, _time: Time<T>) -> Result<Rotation, Self::Error> {
        Ok(provider.icrf_to_j2000())
    }
}

impl<T, P, R> RotateToIcrf<T, P> for Iau<R>
where
    T: TimeScale + Copy,
    R: TryRotationalElements + Copy,
    P: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        provider.iau_to_icrf(time, *self)
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        provider.icrf_to_iau(time, *self)
    }
}

// ---- CIO branch: ICRF ← CIRF ← TIRF ← ITRF ---------------------------------

impl<T, P> RotateToIcrf<T, P> for Cirf
where
    T: TimeScale + Copy,
    P: RotationProvider<T> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        provider.cirf_to_icrf(time)
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        provider.icrf_to_cirf(time)
    }
}

impl<T, P> RotateToIcrf<T, P> for Tirf
where
    T: TimeScale + Copy,
    P: RotationProvider<T> + TryOffset<T, Tdb> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        Ok(provider
            .tirf_to_cirf(time)?
            .compose(provider.cirf_to_icrf(time)?))
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        Ok(provider
            .icrf_to_cirf(time)?
            .compose(provider.cirf_to_tirf(time)?))
    }
}

impl<T, P> RotateToIcrf<T, P> for Itrf
where
    T: TimeScale + Copy,
    P: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Tdb> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        provider.itrf_to_icrf(time)
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        provider.icrf_to_itrf(time)
    }
}

// ---- equinox branch: ICRF ← MOD ← TOD ← PEF --------------------------------

impl<T, P, C> RotateToIcrf<T, P> for Mod<C>
where
    T: TimeScale + Copy,
    C: IersSystem + Into<ReferenceSystem> + Copy,
    P: RotationProvider<T> + TryOffset<T, Tt>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        provider.mod_to_icrf(time, self.0.into())
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        provider.icrf_to_mod(time, self.0.into())
    }
}

impl<T, P, C> RotateToIcrf<T, P> for Tod<C>
where
    T: TimeScale + Copy,
    C: IersSystem + Into<ReferenceSystem> + Copy,
    P: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        // The convention (and its nutation model) comes from the frame value,
        // so `Tod(Iers2003(B))` genuinely computes the 2000B nutation.
        let sys: ReferenceSystem = self.0.into();
        Ok(provider
            .tod_to_mod(time, sys)?
            .compose(provider.mod_to_icrf(time, sys)?))
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        let sys: ReferenceSystem = self.0.into();
        Ok(provider
            .icrf_to_mod(time, sys)?
            .compose(provider.mod_to_tod(time, sys)?))
    }
}

impl<T, P, C> RotateToIcrf<T, P> for Pef<C>
where
    T: TimeScale + Copy,
    C: IersSystem + Into<ReferenceSystem> + Copy,
    P: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Tdb> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        let sys: ReferenceSystem = self.0.into();
        Ok(provider
            .pef_to_tod(time, sys)?
            .compose(provider.tod_to_mod(time, sys)?)
            .compose(provider.mod_to_icrf(time, sys)?))
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        let sys: ReferenceSystem = self.0.into();
        Ok(provider
            .icrf_to_mod(time, sys)?
            .compose(provider.mod_to_tod(time, sys)?)
            .compose(provider.tod_to_pef(time, sys)?))
    }
}

// ---- TEME: tied to the IAU 1976/FK5 (IERS1996) equinox chain ---------------

impl<T, P> RotateToIcrf<T, P> for Teme
where
    T: TimeScale + Copy,
    P: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Tdb>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        let sys = ReferenceSystem::Iers1996;
        Ok(provider
            .teme_to_tod(time)?
            .compose(provider.tod_to_mod(time, sys)?)
            .compose(provider.mod_to_icrf(time, sys)?))
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        let sys = ReferenceSystem::Iers1996;
        Ok(provider
            .icrf_to_mod(time, sys)?
            .compose(provider.mod_to_tod(time, sys)?)
            .compose(provider.tod_to_teme(time)?))
    }
}

// ---- dynamic dispatch ------------------------------------------------------

impl<T, P> RotateToIcrf<T, P> for DynFrame
where
    T: TimeScale + Copy,
    P: RotationProvider<T> + TryOffset<T, Tt> + TryOffset<T, Tdb> + TryOffset<T, Ut1>,
{
    type Error = RotationError;

    fn rotation_to_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        match *self {
            DynFrame::Icrf => Icrf.rotation_to_icrf(provider, time),
            DynFrame::J2000 => J2000.rotation_to_icrf(provider, time),
            DynFrame::Cirf => Cirf.rotation_to_icrf(provider, time),
            DynFrame::Tirf => Tirf.rotation_to_icrf(provider, time),
            DynFrame::Itrf => Itrf.rotation_to_icrf(provider, time),
            DynFrame::Iau(origin) => Iau::try_new(origin)?.rotation_to_icrf(provider, time),
            DynFrame::Mod(sys) => Mod(sys).rotation_to_icrf(provider, time),
            DynFrame::Tod(sys) => Tod(sys).rotation_to_icrf(provider, time),
            DynFrame::Pef(sys) => Pef(sys).rotation_to_icrf(provider, time),
            DynFrame::Teme => Teme.rotation_to_icrf(provider, time),
        }
    }

    fn rotation_from_icrf(&self, provider: &P, time: Time<T>) -> Result<Rotation, Self::Error> {
        match *self {
            DynFrame::Icrf => Icrf.rotation_from_icrf(provider, time),
            DynFrame::J2000 => J2000.rotation_from_icrf(provider, time),
            DynFrame::Cirf => Cirf.rotation_from_icrf(provider, time),
            DynFrame::Tirf => Tirf.rotation_from_icrf(provider, time),
            DynFrame::Itrf => Itrf.rotation_from_icrf(provider, time),
            DynFrame::Iau(origin) => Iau::try_new(origin)?.rotation_from_icrf(provider, time),
            DynFrame::Mod(sys) => Mod(sys).rotation_from_icrf(provider, time),
            DynFrame::Tod(sys) => Tod(sys).rotation_from_icrf(provider, time),
            DynFrame::Pef(sys) => Pef(sys).rotation_from_icrf(provider, time),
            DynFrame::Teme => Teme.rotation_from_icrf(provider, time),
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_core::glam::DMat3;

    use lox_bodies::DynOrigin;
    use lox_time::time_scales::Tai;

    use crate::iers::{Iau2000Model, Iers2003};
    use crate::providers::DefaultRotationProvider;

    use super::*;

    fn epoch() -> Time<Tt> {
        Time::from_two_part_julian_date(Tt, 2454195.5, 0.500754444444444)
    }

    fn max_abs_diff(a: DMat3, b: DMat3) -> f64 {
        let d = a - b;
        d.x_axis
            .abs()
            .max_element()
            .max(d.y_axis.abs().max_element())
            .max(d.z_axis.abs().max_element())
    }

    #[test]
    fn roundtrip_icrf_itrf() {
        let t = epoch();
        let fwd = DefaultRotationProvider.try_rotation(Icrf, Itrf, t).unwrap();
        let bwd = DefaultRotationProvider.try_rotation(Itrf, Icrf, t).unwrap();
        assert_approx_eq!(fwd.m * bwd.m, DMat3::IDENTITY, atol <= 1e-14);
    }

    #[test]
    fn threads_2000b_model() {
        // The hub route reads the nutation model from the frame value, so 2000A
        // and 2000B genuinely differ (mas-level) instead of collapsing to 2000A.
        let t = epoch();
        let tod_a = DefaultRotationProvider
            .try_rotation(Icrf, Tod(Iers2003(Iau2000Model::A)), t)
            .unwrap();
        let tod_b = DefaultRotationProvider
            .try_rotation(Icrf, Tod(Iers2003(Iau2000Model::B)), t)
            .unwrap();
        assert!(max_abs_diff(tod_a.m, tod_b.m) > 1e-9);
    }

    // ---- mixed concrete <-> DynFrame, served by the blanket impl -----------

    fn tai_j2000() -> Time<Tai> {
        Time::j2000(Tai)
    }

    #[test]
    fn mixed_icrf_to_dynframe() {
        let rot = DefaultRotationProvider
            .try_rotation(Icrf, DynFrame::Icrf, tai_j2000())
            .unwrap();
        assert!(rot.m.abs_diff_eq(DMat3::IDENTITY, 1e-14));
    }

    #[test]
    fn mixed_dynframe_to_icrf() {
        let rot = DefaultRotationProvider
            .try_rotation(DynFrame::Icrf, Icrf, tai_j2000())
            .unwrap();
        assert!(rot.m.abs_diff_eq(DMat3::IDENTITY, 1e-14));
    }

    #[test]
    fn mixed_iau_dynorigin_and_dynframe() {
        let iau_earth = Iau::try_new(DynOrigin::Earth).unwrap();
        let fwd = DefaultRotationProvider
            .try_rotation(Icrf, DynFrame::Iau(DynOrigin::Earth), tai_j2000())
            .unwrap();
        let bwd = DefaultRotationProvider
            .try_rotation(iau_earth, DynFrame::Icrf, tai_j2000())
            .unwrap();
        // Non-trivial body-fixed rotation, with a clean round-trip across the
        // concrete↔dynamic boundary.
        assert!(!fwd.m.abs_diff_eq(DMat3::IDENTITY, 1e-6));
        assert!((fwd.m * bwd.m).abs_diff_eq(DMat3::IDENTITY, 1e-14));
    }
}
