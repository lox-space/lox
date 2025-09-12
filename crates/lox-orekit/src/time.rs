use j4rs::{Instance, InvocationArg, errors::J4RsError};
use lox_time::{
    deltas::TimeDelta,
    julian_dates::JulianDate,
    time_scales::{DynTimeScale, TimeScale, offsets::TryOffset},
};

use crate::{JavaInstance, JavaResult, with_jvm};

const ATTOS_IN_SECOND: i64 = 1_000_000_000_000_000_000;

#[derive(Debug)]
pub enum TimeOffset {
    Valid(i64, i64),
    NaN,
    PosInf,
    NegInf,
}

impl From<f64> for TimeOffset {
    fn from(value: f64) -> Self {
        if value.is_nan() {
            return TimeOffset::NaN;
        }
        if value < i64::MIN as f64 {
            return TimeOffset::NegInf;
        }
        if value > i64::MAX as f64 {
            return TimeOffset::PosInf;
        }
        let seconds = value.round_ties_even();
        let subseconds = value - seconds;
        if subseconds.is_sign_negative() {
            let seconds = seconds as i64 - 1;
            let attoseconds =
                (subseconds * ATTOS_IN_SECOND as f64).round() as i64 + ATTOS_IN_SECOND;
            TimeOffset::Valid(seconds, attoseconds)
        } else {
            let seconds = seconds as i64;
            let attoseconds = (subseconds * ATTOS_IN_SECOND as f64).round() as i64;
            TimeOffset::Valid(seconds, attoseconds)
        }
    }
}

struct LoxTimeScale<T: TimeScale>(pub T);

impl<T> TryFrom<LoxTimeScale<T>> for JavaInstance
where
    T: TimeScale,
{
    type Error = J4RsError;

    fn try_from(scale: LoxTimeScale<T>) -> Result<Self, Self::Error> {
        let scale = scale.0.abbreviation();
        match scale {
            "UT1" => todo!(),
            _ => with_jvm(|jvm| {
                Ok(jvm
                    .invoke_static(
                        "org.orekit.time.TimeScalesFactory",
                        format!("get{}", scale).as_str(),
                        InvocationArg::empty(),
                    )?
                    .into())
            }),
        }
    }
}

impl TryFrom<TimeDelta> for JavaInstance {
    type Error = J4RsError;

    fn try_from(delta: TimeDelta) -> Result<Self, Self::Error> {
        let seconds = InvocationArg::try_from(delta.seconds)?.into_primitive()?;
        let attoseconds = InvocationArg::try_from(delta.subsecond.0 * ATTOS_IN_SECOND as f64)
        todo!()
    }
}

pub struct OrekitOffsetProvider;

impl<T, S> TryOffset<T, S> for OrekitOffsetProvider
where
    T: TimeScale,
    S: TimeScale,
{
    type Error = J4RsError;

    fn try_offset(
        &self,
        origin: T,
        target: S,
        delta: lox_time::deltas::TimeDelta,
    ) -> Result<lox_time::deltas::TimeDelta, Self::Error> {
        let date = with_jvm(|jvm| {
            jvm.create_instance(
                "org.orekit.time.DateComponents",
                &[&(delta.days_since_j2000() as i32).try_into()?],
            )
        });
        let second_in_day = delta.seconds_since_j2000() % 86400.0;
        let time = with_jvm(|jvm| {
            jvm.create_instance(
                "org.orekit.time.TimeComponents",
                &[&second_in_day.try_into()?],
            )
        });
        let origin: JavaInstance = LoxTimeScale(origin).try_into()?;
        let target: JavaInstance = LoxTimeScale(target).try_into()?;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use j4rs::InvocationArg;
    use lox_time::{
        deltas::TimeDelta,
        time_scales::{Tai, Tt, offsets::TryOffset},
    };
    use rstest::rstest;

    use crate::{
        test_helpers::init_orekit,
        time::{OrekitOffsetProvider, TimeOffset},
        with_jvm,
    };

    // #[test]
    // fn test_orekit_tai_tt() {
    //     let offset = OrekitOffsetProvider
    //         .try_offset(Tai, Tt, TimeDelta::default())
    //         .unwrap();
    //     assert_eq!(offset.to_decimal_seconds(), 32.164)
    // }

    #[rstest]
    fn test_time_offset(_init_orekit: ()) {
        println!("{:?}", TimeOffset::from(32.164));
        with_jvm(|jvm| {
            let arg = InvocationArg::try_from(32.164)?.into_primitive()?;
            let offset = jvm.create_instance("org.orekit.time.TimeOffset", &[arg])?;
            let string: String =
                jvm.to_rust(jvm.invoke(&offset, "toString", InvocationArg::empty())?)?;
            println!("{:?}", string);
            Ok(())
        })
        .unwrap()
    }
}
