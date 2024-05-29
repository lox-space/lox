/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::frames::{Icrf, ReferenceFrame};
use crate::states::State;
use lox_bodies::python::PyBody;
use lox_time::python::time::PyTime;
use pyo3::pyclass;

#[pyclass]
pub enum PyBodyfixed {
    Foo,
    Bar,
}

impl ReferenceFrame for PyBodyfixed {
    fn name(&self) -> &str {
        todo!()
    }

    fn abbreviation(&self) -> &str {
        todo!()
    }
}

#[pyclass]
pub struct IcrfState(State<PyTime, PyBody, Icrf>);

#[pyclass]
pub struct BodyfixedState(State<PyTime, PyBody, PyBodyfixed>);
