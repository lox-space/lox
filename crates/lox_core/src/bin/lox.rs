use lox_core::bodies::{MercuryBarycenter, NaifId};

fn main() {
    println!("Hello LOX");
    let m = MercuryBarycenter;
    println!("{}", m.id())
}
