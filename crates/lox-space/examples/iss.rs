use lox_space::prelude::*;

fn main() {
    let date = Date::new(2016, 5, 30).unwrap();
    let time = Time::new(12, 0, 0).unwrap();
    let epoch = Epoch::from_date_and_time(TimeScale::TDB, date, time);
    let position = DVec3::new(6068279.27, -1692843.94, -2516619.18) * 1e-3;
    let velocity = DVec3::new(-660.415582, 5495.938726, -5303.093233) * 1e-3;
    let iss = Cartesian::new(epoch, Earth, position, velocity);

    println!("ISS Orbit for Julian Day {}", iss.epoch().j2000());
    println!("=============================");
    println!("Semi-major axis: {:.3} km", iss.semi_major());
    println!("Eccentricity: {:.6}", iss.eccentricity());
    println!("Inclination: {:.3}°", iss.inclination().to_degrees());
    println!(
        "Longitude of ascending node: {:.3}°",
        iss.ascending_node().to_degrees()
    );
    println!("Argument of perigee: {}°", iss.periapsis_arg().to_degrees());
    println!("True anomaly: {:.3}°", iss.true_anomaly().to_degrees());
}
