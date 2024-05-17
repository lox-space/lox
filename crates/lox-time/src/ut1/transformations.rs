use crate::{
    time_scales::{Tai, Ut1},
    transformations::ToTai,
    Time,
};

use super::DeltaUt1TaiProvider;

pub trait ToUt1<T: DeltaUt1TaiProvider> {
    fn to_ut1(&self, provider: &T) -> Result<Time<Ut1>, T::Error>;
}

impl<T: DeltaUt1TaiProvider> ToUt1<T> for Time<Tai> {
    fn to_ut1(&self, provider: &T) -> Result<Time<Ut1>, T::Error> {
        let delta = provider.delta_ut1_tai(self)?;
        Ok(Time::from_delta(Ut1, self.to_delta() + delta))
    }
}

impl<T: DeltaUt1TaiProvider, U: ToTai> ToUt1<T> for U {
    fn to_ut1(&self, provider: &T) -> Result<Time<Ut1>, <T as DeltaUt1TaiProvider>::Error> {
        self.to_tai().to_ut1(provider)
    }
}

impl Time<Ut1> {
    pub fn to_tai<T: DeltaUt1TaiProvider>(&self, provider: &T) -> Result<Time<Tai>, T::Error> {
        let delta = provider.delta_tai_ut1(self)?;
        Ok(Time::from_delta(Tai, self.to_delta() + delta))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use crate::{
        time,
        transformations::{ToTcb, ToTcg, ToTdb, ToTt},
        ut1::DeltaUt1Tai,
        utc::leap_seconds::BuiltinLeapSeconds,
    };

    use super::*;

    #[test]
    fn test_all_scales_to_ut1() {
        let provider = delta_ut1_tai();

        let tai = time!(Tai, 2024, 5, 17, 12, 13, 14.0).unwrap();
        let exp = tai.to_ut1(provider).unwrap();

        let tt = tai.to_tt();
        let act = tt.to_ut1(provider).unwrap();
        assert_eq!(act, exp);
        let tcg = tai.to_tcg();
        let act = tcg.to_ut1(provider).unwrap();
        assert_eq!(act, exp);
        let tcb = tai.to_tcb();
        let act = tcb.to_ut1(provider).unwrap();
        assert_eq!(act, exp);
        let tdb = tai.to_tdb();
        let act = tdb.to_ut1(provider).unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    fn test_ut1_to_tai() {
        let provider = delta_ut1_tai();
        let expected = time!(Tai, 2024, 5, 17, 12, 13, 14.0).unwrap();
        let actual = expected.to_ut1(provider).unwrap().to_tai(provider).unwrap();
        assert_eq!(expected, actual)
    }

    fn delta_ut1_tai() -> &'static DeltaUt1Tai {
        static PROVIDER: OnceLock<DeltaUt1Tai> = OnceLock::new();
        PROVIDER.get_or_init(|| {
            DeltaUt1Tai::new(
                format!(
                    "{}/../../data/finals2000A.all.csv",
                    env!("CARGO_MANIFEST_DIR")
                ),
                BuiltinLeapSeconds,
            )
            .unwrap()
        })
    }
}
