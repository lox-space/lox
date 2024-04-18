use lox_time::prelude::*;

fn main() {
    let tai: Time<Tai> = Time::default();
    let tt = tai.to_tt();
    let tdb = tai.to_tdb();

    println!("TAI: {}", tai);
    println!("TT: {}", tt);
    println!("TDB: {}", tdb);

    println!("{}", tdb.date());
    println!("{}", tdb.time());
}
