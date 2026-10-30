// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{CoordinateOrigin, PointMass, TryPointMass, UndefinedOriginPropertyError};
use lox_core::elements::{Equinoctial, EquinoctialError};
use lox_frames::{NonQuasiInertialFrameError, QuasiInertial, ReferenceFrame, TryQuasiInertial};
use thiserror::Error;

use super::{CartesianOrbit, EquinoctialOrbit, KeplerianOrbit, ModifiedEquinoctialOrbit, Orbit};

/// Errors that can occur when converting an orbit to equinoctial elements.
#[derive(Debug, Error)]
pub enum ToEquinoctialError {
    /// The origin has no gravitational parameter.
    #[error(transparent)]
    UndefinedOriginProperty(#[from] UndefinedOriginPropertyError),
    /// The reference frame is not quasi-inertial.
    #[error(transparent)]
    NonQuasiInertial(#[from] NonQuasiInertialFrameError),
    /// The state cannot be represented by equinoctial elements.
    #[error(transparent)]
    Equinoctial(#[from] EquinoctialError),
}

/// Errors that can occur when converting an equinoctial orbit to Cartesian position and velocity.
#[derive(Debug, Error)]
pub enum FromEquinoctialError {
    /// The origin has no gravitational parameter.
    #[error(transparent)]
    UndefinedOriginProperty(#[from] UndefinedOriginPropertyError),
    /// The elements could not be converted.
    #[error(transparent)]
    Equinoctial(#[from] EquinoctialError),
}

impl<O, R> EquinoctialOrbit<O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Constructs a new equinoctial orbit in a quasi-inertial frame.
    pub const fn new(equinoctial: Equinoctial, time: lox_time::Time, origin: O, frame: R) -> Self
    where
        R: QuasiInertial,
    {
        Orbit::from_state(equinoctial, time, origin, frame)
    }

    /// Constructs a new equinoctial orbit, checking that the frame is quasi-inertial.
    ///
    /// # Errors
    ///
    /// Returns a [`NonQuasiInertialFrameError`] if the frame is not quasi-inertial.
    pub fn try_new(
        equinoctial: Equinoctial,
        time: lox_time::Time,
        origin: O,
        frame: R,
    ) -> Result<Self, NonQuasiInertialFrameError>
    where
        R: TryQuasiInertial,
    {
        frame.try_quasi_inertial()?;
        Ok(Orbit::from_state(equinoctial, time, origin, frame))
    }

    /// Converts this orbit to Cartesian position and velocity.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if Kepler's equation cannot be solved, see
    /// [`Equinoctial::to_cartesian`].
    pub fn to_cartesian(&self) -> Result<CartesianOrbit<O, R>, EquinoctialError>
    where
        O: Copy + PointMass,
        R: Copy,
    {
        Ok(self.with_state(self.state().to_cartesian(self.gravitational_parameter())?))
    }

    /// Converts this orbit to Cartesian position and velocity.
    ///
    /// # Errors
    ///
    /// Returns a [`FromEquinoctialError`] if the gravitational parameter is undefined or
    /// Kepler's equation cannot be solved, see [`Equinoctial::to_cartesian`].
    pub fn try_to_cartesian(&self) -> Result<CartesianOrbit<O, R>, FromEquinoctialError>
    where
        O: Copy + TryPointMass,
        R: Copy,
    {
        Ok(self.with_state(
            self.state()
                .to_cartesian(self.try_gravitational_parameter()?)?,
        ))
    }

    /// Converts this orbit to Keplerian elements.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if Kepler's equation cannot be solved, see
    /// [`Equinoctial::to_keplerian`].
    pub fn to_keplerian(&self) -> Result<KeplerianOrbit<O, R>, EquinoctialError>
    where
        O: Copy,
        R: Copy,
    {
        Ok(self.with_state(self.state().to_keplerian()?))
    }

    /// Converts this orbit to modified equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if Kepler's equation cannot be solved, see
    /// [`Equinoctial::to_modified_equinoctial`].
    pub fn to_modified_equinoctial(
        &self,
    ) -> Result<ModifiedEquinoctialOrbit<O, R>, EquinoctialError>
    where
        O: Copy,
        R: Copy,
    {
        Ok(self.with_state(self.state().to_modified_equinoctial()?))
    }
}

impl<O, R> CartesianOrbit<O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Converts this orbit to equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if the state cannot be represented by equinoctial
    /// elements, see [`Cartesian::to_equinoctial`](lox_core::coords::Cartesian::to_equinoctial).
    pub fn to_equinoctial(&self) -> Result<EquinoctialOrbit<O, R>, EquinoctialError>
    where
        O: Copy + PointMass,
        R: Copy + QuasiInertial,
    {
        Ok(self.with_state(
            self.state()
                .to_equinoctial(self.gravitational_parameter())?,
        ))
    }

    /// Converts this orbit to equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns a [`ToEquinoctialError`] if the frame is not quasi-inertial, the gravitational
    /// parameter is undefined, or the state cannot be represented by equinoctial elements, see
    /// [`Cartesian::to_equinoctial`](lox_core::coords::Cartesian::to_equinoctial).
    pub fn try_to_equinoctial(&self) -> Result<EquinoctialOrbit<O, R>, ToEquinoctialError>
    where
        O: Copy + TryPointMass,
        R: Copy + TryQuasiInertial,
    {
        self.reference_frame().try_quasi_inertial()?;
        Ok(self.with_state(
            self.state()
                .to_equinoctial(self.try_gravitational_parameter()?)?,
        ))
    }
}

impl<O, R> KeplerianOrbit<O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Converts this orbit to equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if the orbit is not closed.
    pub fn to_equinoctial(&self) -> Result<EquinoctialOrbit<O, R>, EquinoctialError>
    where
        O: Copy,
        R: Copy,
    {
        Ok(self.with_state(self.state().to_equinoctial()?))
    }
}

impl<O, R> ModifiedEquinoctialOrbit<O, R>
where
    O: CoordinateOrigin,
    R: ReferenceFrame,
{
    /// Converts this orbit to equinoctial elements.
    ///
    /// # Errors
    ///
    /// Returns an [`EquinoctialError`] if the orbit is not closed.
    pub fn to_equinoctial(&self) -> Result<EquinoctialOrbit<O, R>, EquinoctialError>
    where
        O: Copy,
        R: Copy,
    {
        Ok(self.with_state(self.state().to_equinoctial()?))
    }
}

impl<O, R> TryFrom<EquinoctialOrbit<O, R>> for CartesianOrbit<O, R>
where
    O: CoordinateOrigin + TryPointMass + Copy,
    R: ReferenceFrame + Copy,
{
    type Error = FromEquinoctialError;

    fn try_from(orbit: EquinoctialOrbit<O, R>) -> Result<Self, Self::Error> {
        orbit.try_to_cartesian()
    }
}

impl<O, R> TryFrom<CartesianOrbit<O, R>> for EquinoctialOrbit<O, R>
where
    O: CoordinateOrigin + TryPointMass + Copy,
    R: ReferenceFrame + TryQuasiInertial + Copy,
{
    type Error = ToEquinoctialError;

    fn try_from(orbit: CartesianOrbit<O, R>) -> Result<Self, Self::Error> {
        orbit.try_to_equinoctial()
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

    fn keplerian(inclination: f64) -> Keplerian {
        Keplerian::builder()
            .with_semi_major_axis(7000.0.km(), 0.01)
            .with_inclination(inclination.deg())
            .with_longitude_of_ascending_node(30.0.deg())
            .with_argument_of_periapsis(60.0.deg())
            .with_true_anomaly(90.0.deg())
            .build()
            .unwrap()
    }

    #[test]
    fn test_conversions_preserve_state() {
        let time = Time::j2000(TimeScale::Tai);
        for inclination in [0.0, 51.6, 98.0, 180.0] {
            let kep = KeplerianOrbit::new(keplerian(inclination), time, Earth, Icrf);
            let expected = kep.to_cartesian();

            let from_cartesian = expected.to_equinoctial().unwrap();
            let from_keplerian = kep.to_equinoctial().unwrap();
            let from_mee = kep
                .to_modified_equinoctial()
                .unwrap()
                .to_equinoctial()
                .unwrap();
            for eq in [from_cartesian, from_keplerian, from_mee] {
                assert_eq!(eq.time(), time);
                for actual in [
                    eq.to_cartesian().unwrap(),
                    eq.to_keplerian().unwrap().to_cartesian(),
                    eq.to_modified_equinoctial().unwrap().to_cartesian(),
                ] {
                    assert_approx_eq!(
                        actual.position(),
                        expected.position(),
                        atol <= 1e-6,
                        rtol <= 1e-10
                    );
                    assert_approx_eq!(
                        actual.velocity(),
                        expected.velocity(),
                        atol <= 1e-9,
                        rtol <= 1e-10
                    );
                }
            }
        }
    }

    #[test]
    fn test_try_from_dynamic() {
        let time = Time::j2000(TimeScale::Tai);
        let cart = KeplerianOrbit::new(keplerian(51.6), time, Earth, Icrf)
            .to_cartesian()
            .into_dynamic();
        let eq: EquinoctialOrbit = cart.try_into().unwrap();
        let back: CartesianOrbit = eq.try_into().unwrap();
        assert_approx_eq!(back.position(), cart.position(), rtol <= 1e-10);
    }

    #[test]
    fn test_try_new_rejects_rotating_frame() {
        let time = Time::j2000(TimeScale::Tai);
        let eq = keplerian(51.6).to_equinoctial().unwrap();
        assert!(EquinoctialOrbit::try_new(eq, time, Origin::from(Earth), Frame::Icrf).is_ok());
        assert!(EquinoctialOrbit::try_new(eq, time, Origin::from(Earth), Frame::Itrf).is_err());
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
            rotating.try_to_equinoctial(),
            Err(ToEquinoctialError::NonQuasiInertial(_))
        ));
    }
}
