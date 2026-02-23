// SPDX-FileCopyrightText: 2023 Angus Morrison <github@angus-morrison.com>
// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_space::bodies::Earth;
use lox_space::core::coords::Cartesian;
use lox_space::frames::Icrf;
use lox_space::orbits::DVec3;
use lox_space::orbits::orbits::CartesianOrbit;
use lox_space::time::julian_dates::JulianDate;
use lox_space::time::time_scales::Tdb;
use lox_space::time::{Time, time};

fn main() {
    let time = time!(Tdb, 2016, 5, 30, 12, 0, 0.0).unwrap();
    let position = DVec3::new(6068.27927, -1692.84394, -2516.61918);
    let velocity = DVec3::new(-0.660415582, 5.495938726, -5.303093233);
    let iss_cartesian =
        CartesianOrbit::new(Cartesian::from_vecs(position, velocity), time, Earth, Icrf);
    let iss = iss_cartesian.to_keplerian();

    let title = format!(
        "ISS Orbit for Julian Day Number {} ({})",
        iss.time().days_since_j2000(),
        time
    );
    println!("{title}");
    println!("{}", "=".repeat(title.len()));
    println!(
        "Semi-major axis: {:.3} km",
        iss.semi_major_axis().to_kilometers()
    );
    println!("Eccentricity: {:.6}", iss.eccentricity().as_f64());
    println!(
        "Inclination: {:.3}°",
        iss.inclination().as_f64().to_degrees()
    );
    println!(
        "Longitude of ascending node: {:.3}°",
        iss.longitude_of_ascending_node().as_f64().to_degrees()
    );
    println!(
        "Argument of perigee: {}°",
        iss.argument_of_periapsis().as_f64().to_degrees()
    );
    println!(
        "True anomaly: {:.3}°",
        iss.true_anomaly().as_f64().to_degrees()
    );
}
