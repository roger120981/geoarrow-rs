use std::sync::Arc;

use crate::ffi::from_python::AnyGeometryInput;
use crate::util::{return_array, return_chunked_array};
use geoarrow::algorithm::geo::GeodesicArea;
use pyo3::prelude::*;
use pyo3_arrow::{PyArray, PyChunkedArray};
use pyo3_geoarrow::PyGeoArrowResult;

#[pyfunction]
pub fn geodesic_perimeter(py: Python, input: AnyGeometryInput) -> PyGeoArrowResult<PyObject> {
    match input {
        AnyGeometryInput::Array(arr) => {
            let out = arr.as_ref().geodesic_perimeter()?;
            return_array(py, PyArray::from_array_ref(Arc::new(out)))
        }
        AnyGeometryInput::Chunked(arr) => {
            let out = arr.as_ref().geodesic_perimeter()?;
            return_chunked_array(py, PyChunkedArray::from_array_refs(out.chunk_refs())?)
        }
    }
}