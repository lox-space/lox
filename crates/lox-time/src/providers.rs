#[macro_export]
macro_rules! offset_provider {
    ($provider:ident) => {
        offset_provider!($provider, $crate::offsets::MissingEopProviderError);
        offset_provider!(@ut1 $provider, [
            $crate::time_scales::Tai,
            $crate::time_scales::Tcb,
            $crate::time_scales::Tcg,
            $crate::time_scales::Tdb,
            $crate::time_scales::Tt
        ]);
    };
    ($provider:ident, $error:path) => {
        impl $crate::offsets::OffsetProvider for $provider {}
        offset_provider!(@dyn $provider, $error, [
            $crate::time_scales::Tai,
            $crate::time_scales::Tcb,
            $crate::time_scales::Tcg,
            $crate::time_scales::Tdb,
            $crate::time_scales::Tt,
            $crate::time_scales::Ut1
        ]);
    };
    (@ut1 $provider:ident, [$($scale:path),*]) => {
        $(
            impl $crate::offsets::TryOffset<$crate::time_scales::Ut1, $scale> for $provider
            {
                type Error = $crate::offsets::MissingEopProviderError;

                fn try_offset(
                    &self,
                    _origin: $crate::time_scales::Ut1,
                    _target: $scale,
                    _delta: $crate::deltas::TimeDelta,
                ) -> Result<$crate::deltas::TimeDelta, Self::Error> {
                    Err($crate::offsets::MissingEopProviderError)
                }
            }

            impl $crate::offsets::TryOffset<$scale, $crate::time_scales::Ut1> for $provider
            {
                type Error = $crate::offsets::MissingEopProviderError;

                fn try_offset(
                    &self,
                    _origin: $scale,
                    _target: $crate::time_scales::Ut1,
                    _delta: $crate::deltas::TimeDelta,
                ) -> Result<$crate::deltas::TimeDelta, Self::Error> {
                    Err($crate::offsets::MissingEopProviderError)
                }
            }
        )*
    };
    (@dyn $provider:ident, $error:path, [$($scale:path),*]) => {
        impl $crate::offsets::TryOffset<$crate::time_scales::DynTimeScale, $crate::time_scales::DynTimeScale> for $provider {
            type Error = $error;

            fn try_offset(
                &self,
                origin: $crate::time_scales::DynTimeScale,
                target: $crate::time_scales::DynTimeScale,
                delta: $crate::deltas::TimeDelta,
            ) -> Result<$crate::deltas::TimeDelta, Self::Error> {
                match (origin, target) {
                    ($crate::time_scales::DynTimeScale::Tai, $crate::time_scales::DynTimeScale::Tcb) => {
                        Ok(self.try_offset($crate::time_scales::Tai, $crate::time_scales::Tcb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tai, $crate::time_scales::DynTimeScale::Tcg) =>
                    {
                        Ok(self.try_offset($crate::time_scales::Tai, $crate::time_scales::Tcg, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tai, $crate::time_scales::DynTimeScale::Tdb) => {
                        Ok(self.try_offset($crate::time_scales::Tai, $crate::time_scales::Tdb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tai, $crate::time_scales::DynTimeScale::Tt) => {
                        Ok(self.try_offset($crate::time_scales::Tai, $crate::time_scales::Tt, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcb, $crate::time_scales::DynTimeScale::Tai) => {
                        Ok(self.try_offset($crate::time_scales::Tcb, $crate::time_scales::Tai, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcb, $crate::time_scales::DynTimeScale::Tcg) => {
                        Ok(self.try_offset($crate::time_scales::Tcb, $crate::time_scales::Tcg, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcb, $crate::time_scales::DynTimeScale::Tdb) => {
                        Ok(self.try_offset($crate::time_scales::Tcb, $crate::time_scales::Tdb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcb, $crate::time_scales::DynTimeScale::Tt) => {
                        Ok(self.try_offset($crate::time_scales::Tcb, $crate::time_scales::Tt, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcg, $crate::time_scales::DynTimeScale::Tai) => {
                        Ok(self.try_offset($crate::time_scales::Tcg, $crate::time_scales::Tai, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcg, $crate::time_scales::DynTimeScale::Tcb) => {
                        Ok(self.try_offset($crate::time_scales::Tcg, $crate::time_scales::Tcb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcg, $crate::time_scales::DynTimeScale::Tdb) => {
                        Ok(self.try_offset($crate::time_scales::Tcg, $crate::time_scales::Tdb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcg, $crate::time_scales::DynTimeScale::Tt) => {
                        Ok(self.try_offset($crate::time_scales::Tcg, $crate::time_scales::Tt, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tdb, $crate::time_scales::DynTimeScale::Tai) => {
                        Ok(self.try_offset($crate::time_scales::Tdb, $crate::time_scales::Tai, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tdb, $crate::time_scales::DynTimeScale::Tcb) => {
                        Ok(self.try_offset($crate::time_scales::Tdb, $crate::time_scales::Tcb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tdb, $crate::time_scales::DynTimeScale::Tcg) => {
                        Ok(self.try_offset($crate::time_scales::Tdb, $crate::time_scales::Tcg, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tdb, $crate::time_scales::DynTimeScale::Tt) => {
                        Ok(self.try_offset($crate::time_scales::Tdb, $crate::time_scales::Tt, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tt, $crate::time_scales::DynTimeScale::Tai) => {
                        Ok(self.try_offset($crate::time_scales::Tt, $crate::time_scales::Tai, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tt, $crate::time_scales::DynTimeScale::Tcb) => {
                        Ok(self.try_offset($crate::time_scales::Tt, $crate::time_scales::Tcb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tt, $crate::time_scales::DynTimeScale::Tcg) => {
                        Ok(self.try_offset($crate::time_scales::Tt, $crate::time_scales::Tcg, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tt, $crate::time_scales::DynTimeScale::Tdb) => {
                        Ok(self.try_offset($crate::time_scales::Tt, $crate::time_scales::Tdb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tai, $crate::time_scales::DynTimeScale::Ut1) => {
                        Ok(self.try_offset($crate::time_scales::Tai, $crate::time_scales::Ut1, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcb, $crate::time_scales::DynTimeScale::Ut1) => {
                        Ok(self.try_offset($crate::time_scales::Tcb, $crate::time_scales::Ut1, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tcg, $crate::time_scales::DynTimeScale::Ut1) => {
                        Ok(self.try_offset($crate::time_scales::Tcg, $crate::time_scales::Ut1, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tdb, $crate::time_scales::DynTimeScale::Ut1) => {
                        Ok(self.try_offset($crate::time_scales::Tdb, $crate::time_scales::Ut1, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Tt, $crate::time_scales::DynTimeScale::Ut1) => {
                        Ok(self.try_offset($crate::time_scales::Tt, $crate::time_scales::Ut1, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Ut1, $crate::time_scales::DynTimeScale::Tai) => {
                        Ok(self.try_offset($crate::time_scales::Ut1, $crate::time_scales::Tai, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Ut1, $crate::time_scales::DynTimeScale::Tcb) => {
                        Ok(self.try_offset($crate::time_scales::Ut1, $crate::time_scales::Tcb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Ut1, $crate::time_scales::DynTimeScale::Tcg) => {
                        Ok(self.try_offset($crate::time_scales::Ut1, $crate::time_scales::Tcg, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Ut1, $crate::time_scales::DynTimeScale::Tdb) => {
                        Ok(self.try_offset($crate::time_scales::Ut1, $crate::time_scales::Tdb, delta)?)
                    }
                    ($crate::time_scales::DynTimeScale::Ut1, $crate::time_scales::DynTimeScale::Tt) => {
                        Ok(self.try_offset($crate::time_scales::Ut1, $crate::time_scales::Tt, delta)?)
                    }
                    (_, _) => Ok($crate::deltas::TimeDelta::default()),
                }
            }
        }

        $(
            impl $crate::offsets::TryOffset<$crate::time_scales::DynTimeScale, $scale> for $provider
            {
                type Error = $error;

                fn try_offset(
                    &self,
                    origin: $crate::time_scales::DynTimeScale,
                    target: $scale,
                    delta: $crate::deltas::TimeDelta,
                ) -> Result<$crate::deltas::TimeDelta, Self::Error> {
                    match origin {
                        $crate::time_scales::DynTimeScale::Tai => Ok(self.try_offset($crate::time_scales::Tai, target, delta)?),
                        $crate::time_scales::DynTimeScale::Tcb => Ok(self.try_offset($crate::time_scales::Tcb, target, delta)?),
                        $crate::time_scales::DynTimeScale::Tcg => Ok(self.try_offset($crate::time_scales::Tcg, target, delta)?),
                        $crate::time_scales::DynTimeScale::Tdb => Ok(self.try_offset($crate::time_scales::Tdb, target, delta)?),
                        $crate::time_scales::DynTimeScale::Tt => Ok(self.try_offset($crate::time_scales::Tt, target, delta)?),
                        $crate::time_scales::DynTimeScale::Ut1 => Ok(self.try_offset($crate::time_scales::Ut1, target, delta)?),
                    }
                }
            }

            impl $crate::offsets::TryOffset<$scale, $crate::time_scales::DynTimeScale> for $provider
            {
                type Error = $error;

                fn try_offset(
                    &self,
                    origin: $scale,
                    target: $crate::time_scales::DynTimeScale,
                    delta: $crate::deltas::TimeDelta,
                ) -> Result<$crate::deltas::TimeDelta, Self::Error> {
                    match target {
                        $crate::time_scales::DynTimeScale::Tai => Ok(self.try_offset(origin, $crate::time_scales::Tai, delta)?),
                        $crate::time_scales::DynTimeScale::Tcb => Ok(self.try_offset(origin, $crate::time_scales::Tcb, delta)?),
                        $crate::time_scales::DynTimeScale::Tcg => Ok(self.try_offset(origin, $crate::time_scales::Tcg, delta)?),
                        $crate::time_scales::DynTimeScale::Tdb => Ok(self.try_offset(origin, $crate::time_scales::Tdb, delta)?),
                        $crate::time_scales::DynTimeScale::Tt => Ok(self.try_offset(origin, $crate::time_scales::Tt, delta)?),
                        $crate::time_scales::DynTimeScale::Ut1 => Ok(self.try_offset(origin, $crate::time_scales::Ut1, delta)?),
                    }
                }
            }
        )*
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultOffsetProvider;

offset_provider!(DefaultOffsetProvider);

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::offsets::TryOffset;
    use crate::time_scales::DynTimeScale;
    use crate::{DynTime, calendar_dates::Date, deltas::ToDelta, time_of_day::TimeOfDay};
    use lox_math::assert_close;
    use lox_math::is_close::IsClose;

    const DEFAULT_TOL: f64 = 1e-7;
    const TCB_TOL: f64 = 1e-5;

    // Reference values from Orekit
    //
    // Since we use different algorithms for TCB and UT1 we need to
    // adjust the tolerances accordingly.
    //
    #[rstest]
    #[case::tai_tai("TAI", "TAI", 0.0, None)]
    #[case::tai_tcb("TAI", "TCB", 55.66851419888016, Some(TCB_TOL))]
    #[case::tai_tcg("TAI", "TCG", 33.239589335894145, None)]
    #[case::tai_tdb("TAI", "TDB", 32.183882324981056, None)]
    #[case::tai_tt("TAI", "TT", 32.184, None)]
    #[case::tcb_tai("TCB", "TAI", -55.668513317090046, Some(TCB_TOL))]
    #[case::tcb_tcb("TCB", "TCB", 0.0, Some(TCB_TOL))]
    #[case::tcb_tcg("TCB", "TCG", -22.4289240199929, Some(TCB_TOL))]
    #[case::tcb_tdb("TCB", "TDB", -23.484631010747805, Some(TCB_TOL))]
    #[case::tcb_tt("TCB", "TT", -23.484513317090048, Some(TCB_TOL))]
    #[case::tcg_tai("TCG", "TAI", -33.23958931272851, None)]
    #[case::tcg_tcb("TCG", "TCB", 22.428924359636042, Some(TCB_TOL))]
    #[case::tcg_tcg("TCG", "TCG", 0.0, None)]
    #[case::tcg_tdb("TCG", "TDB", -1.0557069988766656, None)]
    #[case::tcg_tt("TCG", "TT", -1.0555893127285145, None)]
    #[case::tdb_tai("TDB", "TAI", -32.18388231420531, None)]
    #[case::tdb_tcb("TDB", "TCB", 23.48463137488165, Some(TCB_TOL))]
    #[case::tdb_tcg("TDB", "TCG", 1.0557069992589518, None)]
    #[case::tdb_tdb("TDB", "TDB", 0.0, None)]
    #[case::tdb_tt("TDB", "TT", 1.176857946845189E-4, None)]
    #[case::tt_tai("TT", "TAI", -32.184, None)]
    #[case::tt_tcb("TT", "TCB", 23.484513689085105, Some(TCB_TOL))]
    #[case::tt_tcg("TT", "TCG", 1.055589313464182, None)]
    #[case::tt_tdb("TT", "TDB", -1.1768579472004603E-4, None)]
    #[case::tt_tt("TT", "TT", 0.0, None)]
    fn test_dyn_time_scale_offsets_new(
        #[case] scale1: &str,
        #[case] scale2: &str,
        #[case] exp: f64,
        #[case] tol: Option<f64>,
    ) {
        let provider = &DefaultOffsetProvider;
        let scale1: DynTimeScale = scale1.parse().unwrap();
        let scale2: DynTimeScale = scale2.parse().unwrap();
        let date = Date::new(2024, 12, 30).unwrap();
        let time = TimeOfDay::from_hms(10, 27, 13.145).unwrap();
        let dt = DynTime::from_date_and_time(scale1, date, time)
            .unwrap()
            .to_delta();
        let act = provider
            .try_offset(scale1, scale2, dt)
            .unwrap()
            .to_decimal_seconds();
        assert_close!(act, exp, 1e-7, tol.unwrap_or(DEFAULT_TOL));
    }
}
