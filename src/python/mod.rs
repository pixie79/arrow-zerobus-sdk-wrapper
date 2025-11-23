//! Python bindings for Zerobus SDK Wrapper
//!
//! This module provides PyO3 bindings to expose the Rust SDK to Python applications.
//! Supports Python 3.11+ with zero-copy Arrow data transfer via PyArrow.

pub mod bindings;

use pyo3::prelude::*;

/// Python module definition
///
/// This function is called by Python to initialize the module.
#[pymodule]
#[pyo3(name = "arrow_zerobus_sdk_wrapper")]
fn arrow_zerobus_sdk_wrapper(_py: Python, m: &PyModule) -> PyResult<()> {
    bindings::register_module(_py, m)?;
    Ok(())
}
