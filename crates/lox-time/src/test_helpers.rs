use std::sync::OnceLock;

use crate::{ut1::DeltaUt1Tai, utc::leap_seconds::BuiltinLeapSeconds};

pub fn delta_ut1_tai() -> &'static DeltaUt1Tai {
    static PROVIDER: OnceLock<DeltaUt1Tai> = OnceLock::new();
    PROVIDER.get_or_init(|| {
        DeltaUt1Tai::new(
            format!(
                "{}/../../data/finals2000A.all.csv",
                env!("CARGO_MANIFEST_DIR")
            ),
            &BuiltinLeapSeconds,
        )
        .unwrap()
    })
}
