// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "python")]

use std::io;

use lox_core::math::roots::{LoxError, RootFinderError};
use lox_frames::rotations::RotationError;
use lox_space::math::python::{callback_pyerr, raised_exception};
use pyo3::Python;
use pyo3::exceptions::PyValueError;

#[test]
fn test_callback_pyerr_reraises_original_exception() {
    Python::attach(|py| {
        let exc = PyValueError::new_err("boom");
        // Normalize before wrapping so the exception instance exists and
        // identity can be asserted after the round trip.
        let original = exc.value(py).clone().unbind();
        let err = RootFinderError::from(LoxError::new(exc));
        let recovered = callback_pyerr(py, &err);
        assert!(recovered.is_instance_of::<PyValueError>(py));
        assert!(
            recovered.value(py).as_ptr() == original.as_ptr(),
            "the original exception object must be re-raised, not a copy"
        );
    });
}

#[test]
fn test_callback_pyerr_preserves_rust_error_chain() {
    Python::attach(|py| {
        let rot = RotationError::eop(io::Error::other("no EOP data for epoch"));
        let err = RootFinderError::from(LoxError::new(rot));
        assert!(raised_exception(py, &err).is_none());
        let recovered = callback_pyerr(py, &err);
        assert!(recovered.is_instance_of::<PyValueError>(py));
        let msg = recovered.value(py).to_string();
        assert!(msg.contains("EOP error"), "kind missing from: {msg}");
        assert!(
            msg.contains("no EOP data for epoch"),
            "cause missing from: {msg}"
        );
    });
}
