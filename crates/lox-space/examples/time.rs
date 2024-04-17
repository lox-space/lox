use lox_time::prelude::*;

fn main() {
    let tai: Time<Tai> = Time::default();
    let tt = tai.to_tt();
    let tdb = tai.to_tdb();

    println!("TAI: {}", tai);
    println!("TT: {}", tt);
    println!("TDB: {}", tdb);

    let date = Date::new(2000, 1, 1).unwrap();
    let dt = date.with_hms_utc(12, 0, 0).unwrap();
}
