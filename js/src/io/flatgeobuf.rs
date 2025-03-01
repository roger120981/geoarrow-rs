use std::io::Cursor;

use arrow_array::RecordBatchReader;
use arrow_wasm::Table;
use geoarrow::io::flatgeobuf::{FlatGeobufReaderBuilder, FlatGeobufReaderOptions};
// use parquet_wasm::utils::assert_parquet_file_not_empty;
use wasm_bindgen::prelude::*;

use crate::error::WasmResult;

/// Read a FlatGeobuf file into GeoArrow memory
///
/// Example:
///
/// ```js
/// import { tableFromIPC } from "apache-arrow";
/// // Edit the `parquet-wasm` import as necessary
/// import { readParquet } from "parquet-wasm/node2";
///
/// const resp = await fetch("https://example.com/file.parquet");
/// const parquetUint8Array = new Uint8Array(await resp.arrayBuffer());
/// const arrowUint8Array = readParquet(parquetUint8Array);
/// const arrowTable = tableFromIPC(arrowUint8Array);
/// ```
///
/// @param file Uint8Array containing FlatGeobuf data
/// @returns Uint8Array containing Arrow data in [IPC Stream format](https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format). To parse this into an Arrow table, pass to `tableFromIPC` in the Arrow JS bindings.
#[wasm_bindgen(js_name = readFlatGeobuf)]
pub fn read_flatgeobuf(file: &[u8], batch_size: Option<usize>) -> WasmResult<Table> {
    // assert_parquet_file_not_empty(parquet_file)?;
    let cursor = Cursor::new(file);
    let options = FlatGeobufReaderOptions {
        batch_size,
        ..Default::default()
    };
    let reader_builder = FlatGeobufReaderBuilder::open(cursor)?;
    let record_batch_reader = reader_builder.read(options)?;
    let schema = record_batch_reader.schema();
    let batches = record_batch_reader.collect::<std::result::Result<_, _>>()?;
    Ok(Table::new(schema, batches))
}
