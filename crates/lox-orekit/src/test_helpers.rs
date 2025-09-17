use std::path::PathBuf;

use rstest::fixture;

#[fixture]
pub fn init_orekit() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data");
    super::init(path).unwrap()
}
