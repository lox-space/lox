// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_space::bodies::Earth;
use lox_space::core::coords::Cartesian;
use lox_space::frames::Frame::Teme;
use lox_space::frames::Icrf;
use lox_space::frames::providers::DefaultRotationProvider;
use lox_space::frames::rotations::TryRotation;
use lox_space::orbits::CartesianOrbit;
use lox_space::orbits::DVec3;
use lox_space::time;
use lox_space::time::time_scales::Tdb;

fn main() {
    let time = time!(Tdb, 2016, 5, 30, 12, 0, 0.0).unwrap();
    let position = DVec3::new(6068279.27, -1692843.94, -2516619.18);
    let velocity = DVec3::new(-660.415582, 5495.938726, -5303.093233);
    let iss_icrf = CartesianOrbit::new(
        Cartesian::from_vecs(position, velocity),
        time.into_dynamic(),
        Earth,
        Icrf,
    );

    // High-Level Transformation
    let iss_teme = iss_icrf
        .try_to_frame(Teme, &DefaultRotationProvider)
        .unwrap();

    // Low-Level Transformation
    let rot = DefaultRotationProvider
        .try_rotation(Icrf, Teme, time)
        .unwrap();
    let (p_teme, v_teme) = rot.rotate_state(position, velocity);

    let iss_teme2 = CartesianOrbit::new(
        Cartesian::from_vecs(p_teme, v_teme),
        time.into_dynamic(),
        Earth,
        Teme,
    );

    assert_eq!(iss_teme, iss_teme2);
}
