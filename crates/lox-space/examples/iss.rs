/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_bodies::Earth;
use lox_coords::frames::Icrf;
use lox_coords::two_body::{Cartesian, Keplerian};
use lox_coords::DVec3;
use lox_time::julian_dates::JulianDate;
use lox_time::time_scales::Tdb;
use lox_time::{time, Time};

fn main() {
    let time = time!(Tdb, 2016, 5, 30, 12, 0, 0.0).unwrap();
    let position = DVec3::new(6068.27927, -1692.84394, -2516.61918);
    let velocity = DVec3::new(-0.660415582, 5.495938726, -5.303093233);
    let iss_cartesian = Cartesian::new(time, Earth, Icrf, position, velocity);
    let iss = Keplerian::from(iss_cartesian);

    println!("ISS Orbit for Julian Day {}", iss.time().days_since_j2000(),);
    println!("=============================");
    println!("Semi-major axis: {:.3} km", iss.semi_major_axis());
    println!("Eccentricity: {:.6}", iss.eccentricity());
    println!("Inclination: {:.3}°", iss.inclination().to_degrees());
    println!(
        "Longitude of ascending node: {:.3}°",
        iss.ascending_node().to_degrees()
    );
    println!(
        "Argument of perigee: {}°",
        iss.periapsis_argument().to_degrees()
    );
    println!("True anomaly: {:.3}°", iss.true_anomaly().to_degrees());
}
