use std::fmt::{Display, Formatter};

use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyErr};

use crate::time_scales::TimeScale;
use crate::transformations::TimeScaleTransformer;
use crate::{
    julian_dates::JulianDate,
    time_scales::{Tai, Tcb, Tcg, Tdb, Tt, Ut1},
    Time,
};

#[pyclass(name = "TimeScale")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PyTimeScale {
    Tai,
    Tcb,
    Tcg,
    Tdb,
    Tt,
    Ut1,
}

#[pymethods]
impl PyTimeScale {
    #[new]
    fn new(name: &str) -> Result<Self, PyErr> {
        match name {
            "TAI" => Ok(PyTimeScale::Tai),
            "TCB" => Ok(PyTimeScale::Tcb),
            "TCG" => Ok(PyTimeScale::Tcg),
            "TDB" => Ok(PyTimeScale::Tdb),
            "TT" => Ok(PyTimeScale::Tt),
            "UT1" => Ok(PyTimeScale::Ut1),
            _ => Err(PyValueError::new_err(format!(
                "invalid timescale: {}",
                name
            ))),
        }
    }

    fn __repr__(&self) -> String {
        format!("TimeScale(\"{}\")", self)
    }

    fn __str__(&self) -> String {
        format!("{}", self)
    }
}

impl TimeScale for PyTimeScale {
    fn abbreviation(&self) -> &'static str {
        match self {
            PyTimeScale::Tai => Tai.abbreviation(),
            PyTimeScale::Tcb => Tcb.abbreviation(),
            PyTimeScale::Tcg => Tcg.abbreviation(),
            PyTimeScale::Tdb => Tdb.abbreviation(),
            PyTimeScale::Tt => Tt.abbreviation(),
            PyTimeScale::Ut1 => Ut1.abbreviation(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            PyTimeScale::Tai => Tai.name(),
            PyTimeScale::Tcb => Tcb.name(),
            PyTimeScale::Tcg => Tcg.name(),
            PyTimeScale::Tdb => Tdb.name(),
            PyTimeScale::Tt => Tt.name(),
            PyTimeScale::Ut1 => Ut1.name(),
        }
    }
}

impl Display for PyTimeScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

#[pyclass(name = "TimeScaleTransformer")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PyTimeScaleTransformer(TimeScaleTransformer);

#[pyclass(name = "Time")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PyTime(Time<PyTimeScale>);

#[pymethods]
impl PyTime {
    fn to_tai(&self, _transformer: Option<PyTimeScaleTransformer>) -> Result<Self, PyErr> {
        match self.0.scale() {
            PyTimeScale::Tai => Ok(*self),
            PyTimeScale::Tcb => {
                let tcb = self.0.override_scale(Tcb);
                let tai = tcb.to_tai().override_scale(PyTimeScale::Tai);
                Ok(PyTime(tai))
            }
            PyTimeScale::Tcg => {
                let tcg = self.0.override_scale(Tcg);
                let tai = tcg.to_tai().override_scale(PyTimeScale::Tai);
                Ok(PyTime(tai))
            }
            PyTimeScale::Tdb => {
                let tdb = self.0.override_scale(Tdb);
                let tai = tdb.to_tai().override_scale(PyTimeScale::Tai);
                Ok(PyTime(tai))
            }
            PyTimeScale::Tt => {
                let tt = self.0.override_scale(Tt);
                let tai = tt.to_tai().override_scale(PyTimeScale::Tai);
                Ok(PyTime(tai))
            }
            PyTimeScale::Ut1 => {
                unimplemented!()
                // if let Some(transformer) = transformer {
                //     let ut1 = self.0.override_scale(Ut1);
                //     let tai = ut1.to_tai(transformer.0).override_scale(PyTimeScale::Tai);
                //     Ok(PyTime(tai))
                // } else {
                //     Err(PyValueError::new_err(
                //         "transformer must be provided when converting from UT1",
                //     ))
                // }
            }
        }
    }
    fn to_tt(&self, _transformer: Option<PyTimeScaleTransformer>) -> Result<Self, PyErr> {
        match self.0.scale() {
            PyTimeScale::Tai => {
                let tai = self.0.override_scale(Tai);
                let tt = tai.to_tt().override_scale(PyTimeScale::Tt);
                Ok(PyTime(tt))
            }
            PyTimeScale::Tcb => {
                let tcb = self.0.override_scale(Tcb);
                let tai = tcb.to_tai();
                let tt = tai.to_tt().override_scale(PyTimeScale::Tt);
                Ok(PyTime(tt))
            }
            PyTimeScale::Tcg => {
                let tcg = self.0.override_scale(Tcg);
                let tai = tcg.to_tai();
                let tt = tai.to_tt().override_scale(PyTimeScale::Tt);
                Ok(PyTime(tt))
            }
            PyTimeScale::Tdb => {
                let tdb = self.0.override_scale(Tdb);
                let tai = tdb.to_tai();
                let tt = tai.to_tt().override_scale(PyTimeScale::Tt);
                Ok(PyTime(tt))
            }
            PyTimeScale::Tt => Ok(*self),
            PyTimeScale::Ut1 => {
                unimplemented!()
                // if let Some(transformer) = transformer {
                //     let ut1 = self.0.override_scale(Ut1);
                //     let tai = ut1.to_tai(transformer.0);
                //     let tt = tai.to_tt().override_scale(PyTimeScale::Tt);
                //     Ok(PyTime(tt))
                // } else {
                //     Err(PyValueError::new_err(
                //         "transformer must be provided when converting from UT1",
                //     ))
                // }
            }
        }
    }
    fn to_tcb(&self, _transformer: Option<PyTimeScaleTransformer>) -> Result<Self, PyErr> {
        match self.0.scale() {
            PyTimeScale::Tai => {
                let tai = self.0.override_scale(Tai);
                let tt = tai.to_tt();
                let tdb = tt.to_tdb();
                let tcb = tdb.to_tcb().override_scale(PyTimeScale::Tcb);
                Ok(PyTime(tcb))
            }
            PyTimeScale::Tcb => Ok(*self),
            PyTimeScale::Tcg => {
                let tcg = self.0.override_scale(Tcg);
                let tt = tcg.to_tt();
                let tdb = tt.to_tdb();
                let tcb = tdb.to_tcb().override_scale(PyTimeScale::Tcb);
                Ok(PyTime(tcb))
            }
            PyTimeScale::Tdb => {
                let tdb = self.0.override_scale(Tdb);
                let tcb = tdb.to_tcb().override_scale(PyTimeScale::Tcb);
                Ok(PyTime(tcb))
            }
            PyTimeScale::Tt => {
                let tt = self.0.override_scale(Tt);
                let tdb = tt.to_tdb();
                let tcb = tdb.to_tcb().override_scale(PyTimeScale::Tcb);
                Ok(PyTime(tcb))
            }
            PyTimeScale::Ut1 => {
                unimplemented!()
                // if let Some(transformer) = transformer {
                //     let ut1 = self.0.override_scale(Ut1);
                //     let tai = ut1.to_tai(transformer.0);
                //     let tt = tai.to_tt();
                //     let tdb = tt.to_tdb();
                //     let tcb = tdb.to_tcb().override_scale(PyTimeScale::Tcb);
                //     Ok(PyTime(tcb))
                // } else {
                //     Err(PyValueError::new_err(
                //         "transformer must be provided when converting from UT1",
                //     ))
                // }
            }
        }
    }
    fn to_tcg(&self, _transformer: Option<PyTimeScaleTransformer>) -> Result<Self, PyErr> {
        match self.0.scale() {
            PyTimeScale::Tai => {
                let tai = self.0.override_scale(Tai);
                let tt = tai.to_tt();
                let tcg = tt.to_tcg().override_scale(PyTimeScale::Tcg);
                Ok(PyTime(tcg))
            }
            PyTimeScale::Tcb => {
                let tcb = self.0.override_scale(Tcb);
                let tdb = tcb.to_tdb();
                let tt = tdb.to_tt();
                let tcg = tt.to_tcg().override_scale(PyTimeScale::Tcg);
                Ok(PyTime(tcg))
            }
            PyTimeScale::Tcg => Ok(*self),
            PyTimeScale::Tdb => {
                let tdb = self.0.override_scale(Tdb);
                let tt = tdb.to_tt();
                let tcg = tt.to_tcg().override_scale(PyTimeScale::Tcg);
                Ok(PyTime(tcg))
            }
            PyTimeScale::Tt => {
                let tt = self.0.override_scale(Tt);
                let tcg = tt.to_tcg().override_scale(PyTimeScale::Tcg);
                Ok(PyTime(tcg))
            }
            PyTimeScale::Ut1 => {
                unimplemented!()
                // if let Some(transformer) = transformer {
                //     let ut1 = self.0.override_scale(Ut1);
                //     let tai = ut1.to_tai(transformer.0);
                //     let tt = tai.to_tt();
                //     let tcg = tt.to_tcg().override_scale(PyTimeScale::Tcg);
                //     Ok(PyTime(tcg))
                // } else {
                //     Err(PyValueError::new_err(
                //         "transformer must be provided when converting from UT1",
                //     ))
                // }
            }
        }
    }
    fn to_tdb(&self, _transformer: Option<PyTimeScaleTransformer>) -> Result<Self, PyErr> {
        match self.0.scale() {
            PyTimeScale::Tai => {
                let tai = self.0.override_scale(Tai);
                let tt = tai.to_tt();
                let tdb = tt.to_tdb().override_scale(PyTimeScale::Tdb);
                Ok(PyTime(tdb))
            }
            PyTimeScale::Tcb => {
                let tcb = self.0.override_scale(Tcb);
                let tdb = tcb.to_tdb().override_scale(PyTimeScale::Tdb);
                Ok(PyTime(tdb))
            }
            PyTimeScale::Tcg => {
                let tcg = self.0.override_scale(Tcg);
                let tt = tcg.to_tt();
                let tdb = tt.to_tdb().override_scale(PyTimeScale::Tdb);
                Ok(PyTime(tdb))
            }
            PyTimeScale::Tdb => Ok(*self),
            PyTimeScale::Tt => {
                let tt = self.0.override_scale(Tt);
                let tdb = tt.to_tdb().override_scale(PyTimeScale::Tdb);
                Ok(PyTime(tdb))
            }
            PyTimeScale::Ut1 => {
                unimplemented!()
                // if let Some(transformer) = transformer {
                //     let ut1 = self.0.override_scale(Ut1);
                //     let tai = ut1.to_tai(transformer.0);
                //     let tt = tai.to_tt();
                //     let tdb = tt.to_tdb().override_scale(PyTimeScale::Tdb);
                //     Ok(PyTime(tdb))
                // } else {
                //     Err(PyValueError::new_err(
                //         "transformer must be provided when converting from UT1",
                //     ))
                // }
            }
        }
    }
    fn to_ut1(&self, _transformer: Option<PyTimeScaleTransformer>) -> Result<Self, PyErr> {
        unimplemented!()
        // if let Some(transformer) = transformer {
        //     match self.0.scale() {
        //         PyTimeScale::Tai => {
        //             let tai = self.0.override_scale(Tai);
        //             let ut1 = tai.to_ut1(transformer.0);
        //             Ok(PyTime(ut1))
        //         }
        //         PyTimeScale::Tcb => {
        //             let tcb = self.0.override_scale(Tcb);
        //             let tdb = tcb.to_tdb();
        //             let tt = tdb.to_tt();
        //             let tai = tt.to_tai();
        //             let ut1 = tai.to_ut1(transformer.0);
        //             Ok(PyTime(ut1))
        //         }
        //         PyTimeScale::Tcg => {
        //             let tcg = self.0.override_scale(Tcg);
        //             let tt = tcg.to_tt();
        //             let tai = tt.to_tai();
        //             let ut1 = tai.to_ut1(transformer.0);
        //             Ok(PyTime(ut1))
        //         }
        //         PyTimeScale::Tdb => {
        //             let tdb = self.0.override_scale(Tdb);
        //             let tt = tdb.to_tt();
        //             let tai = tt.to_tai();
        //             let ut1 = tai.to_ut1(transformer.0);
        //             Ok(PyTime(ut1))
        //         }
        //         PyTimeScale::Tt => {
        //             let tt = self.0.override_scale(Tt);
        //             let tai = tt.to_tai();
        //             let ut1 = tai.to_ut1(transformer.0);
        //             Ok(PyTime(ut1))
        //         }
        //         PyTimeScale::Ut1 => Ok(*self),
        //     }
        // } else {
        //     Err(PyValueError::new_err(
        //         "transformer must be provided when converting from UT1",
        //     ))
        // }
    }
}

impl JulianDate for PyTime {
    fn julian_date(
        &self,
        epoch: crate::julian_dates::Epoch,
        unit: crate::julian_dates::Unit,
    ) -> f64 {
        self.0.julian_date(epoch, unit)
    }

    fn two_part_julian_date(&self) -> (f64, f64) {
        self.0.two_part_julian_date()
    }
}
