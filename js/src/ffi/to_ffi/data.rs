#![allow(non_snake_case)]

use crate::data::*;
use crate::error::WasmResult;
use arrow_wasm::ffi::FFIData;
use geoarrow::ArrayBase;
use wasm_bindgen::prelude::*;

macro_rules! impl_data {
    ($struct_name:ident) => {
        #[wasm_bindgen]
        impl $struct_name {
            /// Export this Data to FFI structs according to the Arrow C Data Interface.
            ///
            /// This method **does not consume** the Data, so you must remember to call `free` to
            /// release the resources. The underlying arrays are reference counted, so this method
            /// does not copy data, it only prevents the data from being released.
            #[wasm_bindgen(js_name = toFFI)]
            pub fn to_ffi(&self) -> WasmResult<FFIData> {
                let field = self.0.extension_field();
                let arr = self.0.to_array_ref();
                Ok(FFIData::from_arrow(arr.as_ref(), &field)?)
            }

            /// Export this Data to FFI structs according to the Arrow C Data Interface.
            ///
            /// This method **does consume** the Data, so the original Data will be
            /// inaccessible after this call. You must still call {@linkcode FFIData.free} after
            /// you've finished using the FFIData.
            #[wasm_bindgen(js_name = intoFFI)]
            pub fn into_ffi(self) -> WasmResult<FFIData> {
                let field = self.0.extension_field();
                let arr = self.0.into_array_ref();
                Ok(FFIData::from_arrow(arr.as_ref(), &field)?)
            }
        }
    };
}

impl_data!(PointData);
impl_data!(LineStringData);
impl_data!(PolygonData);
impl_data!(MultiPointData);
impl_data!(MultiLineStringData);
impl_data!(MultiPolygonData);
impl_data!(GeometryCollectionData);
impl_data!(RectData);
