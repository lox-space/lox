use lox_core::bodies::barycenters::MercuryBarycenter;
use lox_core::bodies::NaifId;

fn main() {
    println!("Hello LOX");
    println!("{}", MercuryBarycenter::id())
}
