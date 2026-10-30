// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{CoordinateOrigin, PointMass, TryPointMass, UndefinedOriginPropertyError};
use lox_core::elements::{KeplerianError, ModifiedEquinoctial, ModifiedEquinoctialError};
use lox_frames::{NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame, TryQuasiInertial};
use thiserror::Error;

use super::{CartesianOrbit, CheckFrame, KeplerianOrbit, ModifiedEquinoctialOrbit, Orbit};

/// Errors that can occur when converting an orbit to modified equinoctial elements.
#[derive(Debug, Error)]
pub enum ToModifiedEquinoctialError {
    /// The origin has no gravitational parameter.
    #[error(transparent)]
    UndefinedOriginProperty(#[from] UndefinedOriginPropertyError),
    /// The reference frame is not quasi-inertial.
    #[error(transparent)]
    NonQuasiInertial(#[from] NonQuasiInertialFrameError),
    /// The state cannot be represented by modified equinoctial elements.
    #[error(transparent)]
    ModifiedEquinoctial(#[from] ModifiedEquinoctialError),
}

impl<O, R> ModifiedEquinoctialOrbit<O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Constructs a new modified equinoctial orbit in a quasi-inertial frame.
    pub const fn new(
        modified_equinoctial: ModifiedEquinoctial,
        time: lox_time::Time,
        origin: O,
        frame: R,
    ) -> Self
    where
        R: QuasiInertial,
    {
        Orbit::from_state(modified_equinoctial, time, origin, frame)
    }

    /// Constructs a new modified equinoctial orbit, checking that the frame is quasi-inertial.
    ///
    /// # Errors
    ///
    /// Returns a [`NonQuasiInertialFrameError`] if the frame is not quasi-inertial.
    pub fn try_new(
        modified_equinoctial: ModifiedEquinoctial,
        time: lox_time::Time,
        origin: O,
        frame: R,
    ) -> Result<Self, NonQuasiInertialFrameError>
    where
        R: TryQuasiInertial,
    {
        ModifiedEquinoctial::check_frame(&frame)?;
        Ok(Orbit::from_state(modified_equinoctial, time, origin, frame))
    }

    /// Converts this orbit to Cartesian position and velocity.
    pub fn to_cartesian(&self) -> CartesianOrbit<O, R>
    where
        O: Copy + PointMass,
        R: Copy,
    {
        self.with_state(self.state().to_cartesian(self.gravitational_parameter()))
    }

    /// Converts this orbit to Cartesian position and velocity.
    ///
    /// # Errors
    ///
    /// Returns an [`UndefinedOriginPropertyError`] if the gravitational parameter is undefined.
    pub fn try_to_cartesian(&self) -> Result<CartesianOrbit<O, R>, UndefinedOriginPropertyError>
    where
        O: Copy + TryPointMass,
        R: Copy,
    {
        Ok(self.with_state(
            self.state()
                .to_cartesian(self.try_gravitational_parameter()?),
        ))
    }

    /// Converts this orbit to Keplerian elements.
    ///
    /// # Errors
    ///
    /// Returns a [`KeplerianError`] if the elements are rejected by the validation of
    /// [`Keplerian`](lox_core::elements::Keplerian), see
    /// [`ModifiedEquinoctial::to_keplerian`].
    pub fn to_keplerian(&self) -> Result<KeplerianOrbit<O, R>, KeplerianError>
    where
        O: Copy,
        R: Copy,
    {
        Ok(self.with_state(self.state().to_keplerian()?))
    }
}

impl<O, R> CartesianOrbit<O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Converts this orbit to modified equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns a [`ModifiedEquinoctialError`] if the state is not physical, see
    /// [`Cartesian::is_physical`](lox_core::coords::Cartesian::is_physical), or its angular
    /// momentum is zero.
    pub fn to_modified_equinoctial(
        &self,
    ) -> Result<ModifiedEquinoctialOrbit<O, R>, ModifiedEquinoctialError>
    where
        O: Copy + PointMass,
        R: Copy + QuasiInertial,
    {
        Ok(self.with_state(
            self.state()
                .to_modified_equinoctial(self.gravitational_parameter())?,
        ))
    }

    /// Converts this orbit to modified equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns a [`ToModifiedEquinoctialError`] if the frame is not quasi-inertial, the
    /// gravitational parameter is undefined, the state is not physical, see
    /// [`Cartesian::is_physical`](lox_core::coords::Cartesian::is_physical), or its angular
    /// momentum is zero.
    pub fn try_to_modified_equinoctial(
        &self,
    ) -> Result<ModifiedEquinoctialOrbit<O, R>, ToModifiedEquinoctialError>
    where
        O: Copy + TryPointMass,
        R: Copy + TryQuasiInertial,
    {
        ModifiedEquinoctial::check_frame(&self.reference_frame())?;
        Ok(self.with_state(
            self.state()
                .to_modified_equinoctial(self.try_gravitational_parameter()?)?,
        ))
    }
}

impl<O, R> KeplerianOrbit<O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Converts this orbit to modified equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns a [`ModifiedEquinoctialError`] if the true anomaly of an open orbit lies so close
    /// to its asymptotes that the elements are rejected, see
    /// [`Keplerian::to_modified_equinoctial`](lox_core::elements::Keplerian::to_modified_equinoctial).
    pub fn to_modified_equinoctial(
        &self,
    ) -> Result<ModifiedEquinoctialOrbit<O, R>, ModifiedEquinoctialError>
    where
        O: Copy,
        R: Copy,
    {
        Ok(self.with_state(self.state().to_modified_equinoctial()?))
    }
}

impl<O, R> TryFrom<ModifiedEquinoctialOrbit<O, R>> for CartesianOrbit<O, R>
where
    O: CoordinateOrigin + TryPointMass + Copy,
    R: ReferenceFrame + Copy,
{
    type Error = UndefinedOriginPropertyError;

    fn try_from(orbit: ModifiedEquinoctialOrbit<O, R>) -> Result<Self, Self::Error> {
        orbit.try_to_cartesian()
    }
}

impl<O, R> TryFrom<CartesianOrbit<O, R>> for ModifiedEquinoctialOrbit<O, R>
where
    O: CoordinateOrigin + TryPointMass + Copy,
    R: ReferenceFrame + TryQuasiInertial + Copy,
{
    type Error = ToModifiedEquinoctialError;

    fn try_from(orbit: CartesianOrbit<O, R>) -> Result<Self, Self::Error> {
        orbit.try_to_modified_equinoctial()
    }
}

#[cfg(test)]
mod tests {
    use lox_approx::assert_approx_eq;
    use lox_bodies::{Earth, Origin};
    use lox_core::elements::Keplerian;
    use lox_core::units::{AngleUnits, DistanceUnits};
    use lox_frames::{Frame, Icrf};
    use lox_time::Time;
    use lox_time::time_scales::TimeScale;

    use super::*;

    #[test]
    fn test_conversions_preserve_state() {
        let time = Time::j2000(TimeScale::Tai);
        // A hyperbolic orbit, which equinoctial elements cannot represent.
        let kep = Keplerian::builder()
            .with_semi_parameter(20000.0.km(), 1.5)
            .with_inclination(120.0.deg())
            .with_longitude_of_ascending_node(30.0.deg())
            .with_argument_of_periapsis(60.0.deg())
            .with_true_anomaly(45.0.deg())
            .build()
            .unwrap();
        let kep = KeplerianOrbit::new(kep, time, Earth, Icrf);
        let expected = kep.to_cartesian();

        for mee in [
            expected.to_modified_equinoctial().unwrap(),
            kep.to_modified_equinoctial().unwrap(),
        ] {
            assert_eq!(mee.time(), time);
            assert!(mee.to_equinoctial().is_err());
            for actual in [
                mee.to_cartesian(),
                mee.to_keplerian().unwrap().to_cartesian(),
            ] {
                assert_approx_eq!(actual.position(), expected.position(), rtol <= 1e-10);
                assert_approx_eq!(actual.velocity(), expected.velocity(), rtol <= 1e-10);
            }
        }
    }

    #[test]
    fn test_try_from_dynamic() {
        let time = Time::j2000(TimeScale::Tai);
        let kep = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.01)
            .with_inclination(51.6.deg())
            .build()
            .unwrap();
        let cart = KeplerianOrbit::new(kep, time, Earth, Icrf)
            .to_cartesian()
            .into_dynamic();
        let mee: ModifiedEquinoctialOrbit = cart.try_into().unwrap();
        let back: CartesianOrbit = mee.try_into().unwrap();
        assert_approx_eq!(back.position(), cart.position(), rtol <= 1e-10);
    }

    #[test]
    fn test_try_new_rejects_rotating_frame() {
        let time = Time::j2000(TimeScale::Tai);
        let kep = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.01)
            .build()
            .unwrap();
        let mee = kep.to_modified_equinoctial().unwrap();
        let earth = Origin::from(Earth);
        assert!(ModifiedEquinoctialOrbit::try_new(mee, time, earth, Frame::Icrf).is_ok());
        assert!(ModifiedEquinoctialOrbit::try_new(mee, time, earth, Frame::Itrf).is_err());
    }

    #[test]
    fn test_try_to_rejects_rotating_frame() {
        let time = Time::j2000(TimeScale::Tai);
        let kep = Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.01)
            .with_inclination(51.6.deg())
            .build()
            .unwrap();
        let cart = KeplerianOrbit::new(kep, time, Earth, Icrf).to_cartesian();
        let rotating =
            CartesianOrbit::from_state(cart.state(), time, Origin::from(Earth), Frame::Itrf);
        assert!(matches!(
            rotating.try_to_modified_equinoctial(),
            Err(ToModifiedEquinoctialError::NonQuasiInertial(_))
        ));
    }
}
