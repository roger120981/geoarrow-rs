use crate::algorithm::geo::utils::zeroes;
use crate::algorithm::native::Unary;
use crate::array::*;
use crate::chunked_array::{ChunkedArray, ChunkedGeometryArray, ChunkedGeometryArrayTrait};
use crate::datatypes::{Dimension, GeoDataType};
use crate::error::{GeoArrowError, Result};
use crate::trait_::GeometryScalarTrait;
use crate::GeometryArrayTrait;
use arrow_array::{Float64Array, OffsetSizeTrait};
use geo::HaversineLength as _HaversineLength;

/// Determine the length of a geometry using the [haversine formula].
///
/// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
///
/// *Note*: this implementation uses a mean earth radius of 6371.088 km, based on the [recommendation of
/// the IUGG](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
pub trait HaversineLength {
    type Output;

    /// Determine the length of a geometry using the [haversine formula].
    ///
    /// # Units
    ///
    /// - return value: meters
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::LineString;
    /// use geoarrow::array::LineStringArray;
    /// use geoarrow::algorithm::geo::HaversineLength;
    ///
    /// let linestring = LineString::<f64>::from(vec![
    ///     // New York City
    ///     (-74.006, 40.7128),
    ///     // London
    ///     (-0.1278, 51.5074),
    /// ]);
    /// let linestring_array: LineStringArray<i32, 2> = vec![linestring].as_slice().into();
    ///
    /// let length_array = linestring_array.haversine_length();
    ///
    /// assert_eq!(
    ///     5_570_230., // meters
    ///     length_array.value(0).round()
    /// );
    /// ```
    ///
    /// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
    fn haversine_length(&self) -> Self::Output;
}

// Note: this can't (easily) be parameterized in the macro because PointArray is not generic over O
impl HaversineLength for PointArray<2> {
    type Output = Float64Array;

    fn haversine_length(&self) -> Self::Output {
        zeroes(self.len(), self.nulls())
    }
}

/// Implementation where the result is zero.
macro_rules! zero_impl {
    ($type:ty) => {
        impl<O: OffsetSizeTrait> HaversineLength for $type {
            type Output = Float64Array;

            fn haversine_length(&self) -> Self::Output {
                zeroes(self.len(), self.nulls())
            }
        }
    };
}

zero_impl!(MultiPointArray<O, 2>);

/// Implementation that iterates over geo objects
macro_rules! iter_geo_impl {
    ($type:ty) => {
        impl<O: OffsetSizeTrait> HaversineLength for $type {
            type Output = Float64Array;

            fn haversine_length(&self) -> Self::Output {
                self.unary_primitive(|geom| geom.to_geo().haversine_length())
            }
        }
    };
}

iter_geo_impl!(LineStringArray<O, 2>);
iter_geo_impl!(MultiLineStringArray<O, 2>);

impl HaversineLength for &dyn GeometryArrayTrait {
    type Output = Result<Float64Array>;

    fn haversine_length(&self) -> Self::Output {
        let result = match self.data_type() {
            GeoDataType::Point(_, Dimension::XY) => self.as_point_2d().haversine_length(),
            GeoDataType::LineString(_, Dimension::XY) => {
                self.as_line_string_2d().haversine_length()
            }
            GeoDataType::LargeLineString(_, Dimension::XY) => {
                self.as_large_line_string_2d().haversine_length()
            }
            // GeoDataType::Polygon(_, Dimension::XY) => self.as_polygon_2d().haversine_length(),
            // GeoDataType::LargePolygon(_, Dimension::XY) => self.as_large_polygon_2d().haversine_length(),
            GeoDataType::MultiPoint(_, Dimension::XY) => {
                self.as_multi_point_2d().haversine_length()
            }
            GeoDataType::LargeMultiPoint(_, Dimension::XY) => {
                self.as_large_multi_point_2d().haversine_length()
            }
            GeoDataType::MultiLineString(_, Dimension::XY) => {
                self.as_multi_line_string_2d().haversine_length()
            }
            GeoDataType::LargeMultiLineString(_, Dimension::XY) => {
                self.as_large_multi_line_string_2d().haversine_length()
            }
            // GeoDataType::MultiPolygon(_, Dimension::XY) => self.as_multi_polygon_2d().haversine_length(),
            // GeoDataType::LargeMultiPolygon(_, Dimension::XY) => self.as_large_multi_polygon_2d().haversine_length(),
            // GeoDataType::Mixed(_, Dimension::XY) => self.as_mixed_2d().haversine_length(),
            // GeoDataType::LargeMixed(_, Dimension::XY) => self.as_large_mixed_2d().haversine_length(),
            // GeoDataType::GeometryCollection(_, Dimension::XY) => self.as_geometry_collection_2d().haversine_length(),
            // GeoDataType::LargeGeometryCollection(_, Dimension::XY) => {
            //     self.as_large_geometry_collection_2d().haversine_length()
            // }
            _ => return Err(GeoArrowError::IncorrectType("".into())),
        };
        Ok(result)
    }
}

impl HaversineLength for ChunkedGeometryArray<PointArray<2>> {
    type Output = Result<ChunkedArray<Float64Array>>;

    fn haversine_length(&self) -> Self::Output {
        self.map(|chunk| chunk.haversine_length()).try_into()
    }
}

/// Implementation that iterates over chunks
macro_rules! chunked_impl {
    ($type:ty) => {
        impl<O: OffsetSizeTrait> HaversineLength for $type {
            type Output = Result<ChunkedArray<Float64Array>>;

            fn haversine_length(&self) -> Self::Output {
                self.map(|chunk| chunk.haversine_length()).try_into()
            }
        }
    };
}

chunked_impl!(ChunkedGeometryArray<LineStringArray<O, 2>>);
chunked_impl!(ChunkedGeometryArray<MultiPointArray<O, 2>>);
chunked_impl!(ChunkedGeometryArray<MultiLineStringArray<O, 2>>);

impl HaversineLength for &dyn ChunkedGeometryArrayTrait {
    type Output = Result<ChunkedArray<Float64Array>>;

    fn haversine_length(&self) -> Self::Output {
        match self.data_type() {
            GeoDataType::Point(_, Dimension::XY) => self.as_point_2d().haversine_length(),
            GeoDataType::LineString(_, Dimension::XY) => {
                self.as_line_string_2d().haversine_length()
            }
            GeoDataType::LargeLineString(_, Dimension::XY) => {
                self.as_large_line_string_2d().haversine_length()
            }
            // GeoDataType::Polygon(_, Dimension::XY) => self.as_polygon_2d().haversine_length(),
            // GeoDataType::LargePolygon(_, Dimension::XY) => self.as_large_polygon_2d().haversine_length(),
            GeoDataType::MultiPoint(_, Dimension::XY) => {
                self.as_multi_point_2d().haversine_length()
            }
            GeoDataType::LargeMultiPoint(_, Dimension::XY) => {
                self.as_large_multi_point_2d().haversine_length()
            }
            GeoDataType::MultiLineString(_, Dimension::XY) => {
                self.as_multi_line_string_2d().haversine_length()
            }
            GeoDataType::LargeMultiLineString(_, Dimension::XY) => {
                self.as_large_multi_line_string_2d().haversine_length()
            }
            // GeoDataType::MultiPolygon(_, Dimension::XY) => self.as_multi_polygon_2d().haversine_length(),
            // GeoDataType::LargeMultiPolygon(_, Dimension::XY) => self.as_large_multi_polygon_2d().haversine_length(),
            // GeoDataType::Mixed(_, Dimension::XY) => self.as_mixed_2d().haversine_length(),
            // GeoDataType::LargeMixed(_, Dimension::XY) => self.as_large_mixed_2d().haversine_length(),
            // GeoDataType::GeometryCollection(_, Dimension::XY) => self.as_geometry_collection_2d().haversine_length(),
            // GeoDataType::LargeGeometryCollection(_, Dimension::XY) => {
            //     self.as_large_geometry_collection_2d().haversine_length()
            // }
            _ => Err(GeoArrowError::IncorrectType("".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::array::LineStringArray;
    use arrow_array::Array;
    use geo::line_string;

    #[test]
    fn haversine_length_geoarrow() {
        let input_geom = line_string![
            // New York City
            (x: -74.006, y: 40.7128),
            // London
            (x: -0.1278, y: 51.5074),
        ];
        let input_array: LineStringArray<i64, 2> = vec![input_geom].as_slice().into();
        let result_array = input_array.haversine_length();

        // Meters
        let expected = 5_570_230.0_f64;
        assert_eq!(expected, result_array.value(0).round());
        assert!(result_array.is_valid(0));
    }
}
