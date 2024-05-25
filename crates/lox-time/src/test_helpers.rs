use std::{path::PathBuf, sync::OnceLock};

use crate::{ut1::DeltaUt1Tai, utc::leap_seconds::BuiltinLeapSeconds};

pub fn data_dir() -> PathBuf {
    PathBuf::from(format!("{}/../../data", env!("CARGO_MANIFEST_DIR")))
}

pub fn delta_ut1_tai() -> &'static DeltaUt1Tai {
    static PROVIDER: OnceLock<DeltaUt1Tai> = OnceLock::new();
    PROVIDER.get_or_init(|| {
        DeltaUt1Tai::new(data_dir().join("finals2000A.all.csv"), &BuiltinLeapSeconds).unwrap()
    })
}
