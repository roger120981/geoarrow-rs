use geo::{LineString, line_string};

use crate::array::{LineStringArray, LineStringBuilder};
use geoarrow_schema::{CoordType, Dimension};

pub(crate) fn ls0() -> LineString {
    line_string![
        (x: 0., y: 1.),
        (x: 1., y: 2.)
    ]
}

pub(crate) fn ls1() -> LineString {
    line_string![
        (x: 3., y: 4.),
        (x: 5., y: 6.)
    ]
}

#[allow(dead_code)]
pub(crate) fn ls_array() -> LineStringArray {
    let geoms = vec![ls0(), ls1()];
    LineStringBuilder::from_line_strings(
        &geoms,
        Dimension::XY,
        CoordType::Interleaved,
        Default::default(),
    )
    .finish()
}
