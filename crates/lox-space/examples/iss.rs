/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lox_core::time::utc::UTC;
use lox_space::prelude::*;

fn main() {
    let date = Date::new(2016, 5, 30).unwrap();
    let time = UTC::new(12, 0, 0).unwrap();
    let epoch = ContinuousTime::from_date_and_utc_timestamp(ContinuousTimeScale::TDB, date, time);
    let position = DVec3::new(6068279.27, -1692843.94, -2516619.18) * 1e-3;
    let velocity = DVec3::new(-660.415582, 5495.938726, -5303.093233) * 1e-3;
    let iss_cartesian = Cartesian::new(epoch, Earth, Icrf, position, velocity);
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
