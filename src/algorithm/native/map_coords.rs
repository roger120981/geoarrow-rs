use std::sync::Arc;

use arrow_array::OffsetSizeTrait;

use crate::array::*;
use crate::chunked_array::*;
use crate::datatypes::{Dimension, GeoDataType};
use crate::error::{GeoArrowError, Result};
use crate::geo_traits::{
    GeometryCollectionTrait, GeometryTrait, GeometryType, LineStringTrait, MultiLineStringTrait,
    MultiPointTrait, MultiPolygonTrait, PolygonTrait, RectTrait,
};
use crate::scalar::*;
use crate::trait_::GeometryArrayAccessor;
use crate::GeometryArrayTrait;

pub trait MapCoords {
    type Output;

    fn map_coords<F>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> geo::Coord + Sync,
    {
        self.try_map_coords(|coord| Ok::<_, GeoArrowError>(map_op(coord)))
    }

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>;
}

// Scalar impls

impl MapCoords for Coord<'_, 2> {
    type Output = geo::Coord;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(map_op(self)?)
    }
}

impl MapCoords for Point<'_, 2> {
    type Output = geo::Point;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(geo::Point(map_op(&self.coord())?))
    }
}

impl<O: OffsetSizeTrait> MapCoords for LineString<'_, O, 2> {
    type Output = geo::LineString;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let output_coords = self
            .coords()
            .map(|point| map_op(&point.coord()))
            .collect::<std::result::Result<Vec<_>, E>>()?;
        Ok(geo::LineString::new(output_coords))
    }
}

impl<O: OffsetSizeTrait> MapCoords for Polygon<'_, O, 2> {
    type Output = geo::Polygon;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        if self.exterior().is_none() {
            return Err(GeoArrowError::General(
                "Empty polygons not yet supported in MapCoords".to_string(),
            ));
        }
        let exterior = self.exterior().unwrap().try_map_coords(&map_op)?;
        let interiors = self
            .interiors()
            .map(|int| int.try_map_coords(&map_op))
            .collect::<Result<Vec<_>>>()?;
        Ok(geo::Polygon::new(exterior, interiors))
    }
}

impl<O: OffsetSizeTrait> MapCoords for MultiPoint<'_, O, 2> {
    type Output = geo::MultiPoint;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let points = self
            .points()
            .map(|point| point.try_map_coords(&map_op))
            .collect::<Result<Vec<_>>>()?;
        Ok(geo::MultiPoint::new(points))
    }
}

impl<O: OffsetSizeTrait> MapCoords for MultiLineString<'_, O, 2> {
    type Output = geo::MultiLineString;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let lines = self
            .lines()
            .map(|line_string| line_string.try_map_coords(&map_op))
            .collect::<Result<Vec<_>>>()?;
        Ok(geo::MultiLineString::new(lines))
    }
}

impl<O: OffsetSizeTrait> MapCoords for MultiPolygon<'_, O, 2> {
    // TODO: support empty polygons within a multi polygon
    type Output = geo::MultiPolygon;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let polygons = self
            .polygons()
            .map(|polygon| polygon.try_map_coords(&map_op))
            .collect::<Result<Vec<_>>>()?;
        Ok(geo::MultiPolygon::new(polygons))
    }
}

impl<O: OffsetSizeTrait> MapCoords for Geometry<'_, O, 2> {
    type Output = geo::Geometry;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        match self.as_type() {
            GeometryType::Point(geom) => Ok(geo::Geometry::Point(geom.try_map_coords(&map_op)?)),
            GeometryType::LineString(geom) => {
                Ok(geo::Geometry::LineString(geom.try_map_coords(&map_op)?))
            }
            GeometryType::Polygon(geom) => {
                Ok(geo::Geometry::Polygon(geom.try_map_coords(&map_op)?))
            }
            GeometryType::MultiPoint(geom) => {
                Ok(geo::Geometry::MultiPoint(geom.try_map_coords(&map_op)?))
            }
            GeometryType::MultiLineString(geom) => Ok(geo::Geometry::MultiLineString(
                geom.try_map_coords(&map_op)?,
            )),
            GeometryType::MultiPolygon(geom) => {
                Ok(geo::Geometry::MultiPolygon(geom.try_map_coords(&map_op)?))
            }
            GeometryType::GeometryCollection(geom) => Ok(geo::Geometry::GeometryCollection(
                geom.try_map_coords(&map_op)?,
            )),
            GeometryType::Rect(geom) => Ok(geo::Geometry::Rect(geom.try_map_coords(&map_op)?)),
        }
    }
}

impl<O: OffsetSizeTrait> MapCoords for GeometryCollection<'_, O, 2> {
    type Output = geo::GeometryCollection;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let geoms = self
            .geometries()
            .map(|geom| geom.try_map_coords(&map_op))
            .collect::<Result<Vec<_>>>()?;
        Ok(geo::GeometryCollection::new_from(geoms))
    }
}

impl MapCoords for Rect<'_, 2> {
    type Output = geo::Rect;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let (minx, miny) = self.lower();
        let (maxx, maxy) = self.upper();
        let coords = vec![minx, miny, maxx, maxy];
        let coord_buffer = CoordBuffer::Interleaved(InterleavedCoordBuffer::new(coords.into()));
        let lower_coord = coord_buffer.value(0);
        let upper_coord = coord_buffer.value(1);

        let new_lower = map_op(&lower_coord)?;
        let new_upper = map_op(&upper_coord)?;
        Ok(geo::Rect::new(new_lower, new_upper))
    }
}

impl MapCoords for PointArray<2> {
    type Output = PointArray<2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = PointBuilder::with_capacity_and_options(
            self.buffer_lengths(),
            self.coord_type(),
            self.metadata(),
        );
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.coord().try_map_coords(&map_op)?;
                builder.push_point(Some(&result));
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl<O: OffsetSizeTrait> MapCoords for LineStringArray<O, 2> {
    type Output = LineStringArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = LineStringBuilder::with_capacity_and_options(
            self.buffer_lengths(),
            self.coord_type(),
            self.metadata(),
        );
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.try_map_coords(&map_op)?;
                builder.push_line_string(Some(&result))?;
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl<O: OffsetSizeTrait> MapCoords for PolygonArray<O, 2> {
    type Output = PolygonArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = PolygonBuilder::with_capacity_and_options(
            self.buffer_lengths(),
            self.coord_type(),
            self.metadata(),
        );
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.try_map_coords(&map_op)?;
                builder.push_polygon(Some(&result))?;
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl<O: OffsetSizeTrait> MapCoords for MultiPointArray<O, 2> {
    type Output = MultiPointArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = MultiPointBuilder::with_capacity_and_options(
            self.buffer_lengths(),
            self.coord_type(),
            self.metadata(),
        );
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.try_map_coords(&map_op)?;
                builder.push_multi_point(Some(&result))?;
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl<O: OffsetSizeTrait> MapCoords for MultiLineStringArray<O, 2> {
    type Output = MultiLineStringArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = MultiLineStringBuilder::with_capacity_and_options(
            self.buffer_lengths(),
            self.coord_type(),
            self.metadata(),
        );
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.try_map_coords(&map_op)?;
                builder.push_multi_line_string(Some(&result))?;
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl<O: OffsetSizeTrait> MapCoords for MultiPolygonArray<O, 2> {
    type Output = MultiPolygonArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = MultiPolygonBuilder::with_capacity_and_options(
            self.buffer_lengths(),
            self.coord_type(),
            self.metadata(),
        );
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.try_map_coords(&map_op)?;
                builder.push_multi_polygon(Some(&result))?;
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl<O: OffsetSizeTrait> MapCoords for MixedGeometryArray<O, 2> {
    type Output = MixedGeometryArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = MixedGeometryBuilder::with_capacity_and_options(
            self.buffer_lengths(),
            self.coord_type(),
            self.metadata(),
        );
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.try_map_coords(&map_op)?;
                builder.push_geometry(Some(&result))?;
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl<O: OffsetSizeTrait> MapCoords for GeometryCollectionArray<O, 2> {
    type Output = GeometryCollectionArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = GeometryCollectionBuilder::with_capacity_and_options(
            self.buffer_lengths(),
            self.coord_type(),
            self.metadata(),
        );
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.try_map_coords(&map_op)?;
                builder.push_geometry_collection(Some(&result))?;
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl MapCoords for RectArray<2> {
    type Output = RectArray<2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let mut builder = RectBuilder::with_capacity_and_options(self.len(), self.metadata());
        for maybe_geom in self.iter() {
            if let Some(geom) = maybe_geom {
                let result = geom.try_map_coords(&map_op)?;
                builder.push_rect(Some(&result));
            } else {
                builder.push_null()
            }
        }
        Ok(builder.finish())
    }
}

impl MapCoords for &dyn GeometryArrayTrait {
    type Output = Arc<dyn GeometryArrayTrait>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let result: Arc<dyn GeometryArrayTrait> = match self.data_type() {
            GeoDataType::Point(_, Dimension::XY) => {
                Arc::new(self.as_point_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LineString(_, Dimension::XY) => {
                Arc::new(self.as_line_string_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeLineString(_, Dimension::XY) => {
                Arc::new(self.as_large_line_string_2d().try_map_coords(map_op)?)
            }
            GeoDataType::Polygon(_, Dimension::XY) => {
                Arc::new(self.as_polygon_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargePolygon(_, Dimension::XY) => {
                Arc::new(self.as_large_polygon_2d().try_map_coords(map_op)?)
            }
            GeoDataType::MultiPoint(_, Dimension::XY) => {
                Arc::new(self.as_multi_point_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeMultiPoint(_, Dimension::XY) => {
                Arc::new(self.as_large_multi_point_2d().try_map_coords(map_op)?)
            }
            GeoDataType::MultiLineString(_, Dimension::XY) => {
                Arc::new(self.as_multi_line_string_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeMultiLineString(_, Dimension::XY) => Arc::new(
                self.as_large_multi_line_string_2d()
                    .try_map_coords(map_op)?,
            ),
            GeoDataType::MultiPolygon(_, Dimension::XY) => {
                Arc::new(self.as_multi_polygon_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeMultiPolygon(_, Dimension::XY) => {
                Arc::new(self.as_large_multi_polygon_2d().try_map_coords(map_op)?)
            }
            GeoDataType::Mixed(_, Dimension::XY) => {
                Arc::new(self.as_mixed_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeMixed(_, Dimension::XY) => {
                Arc::new(self.as_large_mixed_2d().try_map_coords(map_op)?)
            }
            GeoDataType::GeometryCollection(_, Dimension::XY) => {
                Arc::new(self.as_geometry_collection_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeGeometryCollection(_, Dimension::XY) => Arc::new(
                self.as_large_geometry_collection_2d()
                    .try_map_coords(map_op)?,
            ),
            GeoDataType::Rect(Dimension::XY) => Arc::new(self.as_rect_2d().try_map_coords(map_op)?),
            _ => return Err(GeoArrowError::IncorrectType("".into())),
        };
        Ok(result)
    }
}

impl MapCoords for ChunkedPointArray<2> {
    type Output = ChunkedPointArray<2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl<O: OffsetSizeTrait> MapCoords for ChunkedLineStringArray<O, 2> {
    type Output = ChunkedLineStringArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl<O: OffsetSizeTrait> MapCoords for ChunkedPolygonArray<O, 2> {
    type Output = ChunkedPolygonArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl<O: OffsetSizeTrait> MapCoords for ChunkedMultiPointArray<O, 2> {
    type Output = ChunkedMultiPointArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl<O: OffsetSizeTrait> MapCoords for ChunkedMultiLineStringArray<O, 2> {
    type Output = ChunkedMultiLineStringArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl<O: OffsetSizeTrait> MapCoords for ChunkedMultiPolygonArray<O, 2> {
    type Output = ChunkedMultiPolygonArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl<O: OffsetSizeTrait> MapCoords for ChunkedMixedGeometryArray<O, 2> {
    type Output = ChunkedMixedGeometryArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl<O: OffsetSizeTrait> MapCoords for ChunkedGeometryCollectionArray<O, 2> {
    type Output = ChunkedGeometryCollectionArray<O, 2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl MapCoords for ChunkedRectArray<2> {
    type Output = ChunkedRectArray<2>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        Ok(ChunkedGeometryArray::new(
            self.try_map(|chunk| chunk.try_map_coords(&map_op))?,
        ))
    }
}

impl MapCoords for &dyn ChunkedGeometryArrayTrait {
    type Output = Arc<dyn ChunkedGeometryArrayTrait>;

    fn try_map_coords<F, E>(&self, map_op: F) -> Result<Self::Output>
    where
        F: Fn(&crate::scalar::Coord<2>) -> std::result::Result<geo::Coord, E> + Sync,
        GeoArrowError: From<E>,
    {
        let result: Arc<dyn ChunkedGeometryArrayTrait> = match self.data_type() {
            GeoDataType::Point(_, Dimension::XY) => {
                Arc::new(self.as_point_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LineString(_, Dimension::XY) => {
                Arc::new(self.as_line_string_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeLineString(_, Dimension::XY) => {
                Arc::new(self.as_large_line_string_2d().try_map_coords(map_op)?)
            }
            GeoDataType::Polygon(_, Dimension::XY) => {
                Arc::new(self.as_polygon_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargePolygon(_, Dimension::XY) => {
                Arc::new(self.as_large_polygon_2d().try_map_coords(map_op)?)
            }
            GeoDataType::MultiPoint(_, Dimension::XY) => {
                Arc::new(self.as_multi_point_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeMultiPoint(_, Dimension::XY) => {
                Arc::new(self.as_large_multi_point_2d().try_map_coords(map_op)?)
            }
            GeoDataType::MultiLineString(_, Dimension::XY) => {
                Arc::new(self.as_multi_line_string_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeMultiLineString(_, Dimension::XY) => Arc::new(
                self.as_large_multi_line_string_2d()
                    .try_map_coords(map_op)?,
            ),
            GeoDataType::MultiPolygon(_, Dimension::XY) => {
                Arc::new(self.as_multi_polygon_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeMultiPolygon(_, Dimension::XY) => {
                Arc::new(self.as_large_multi_polygon_2d().try_map_coords(map_op)?)
            }
            GeoDataType::Mixed(_, Dimension::XY) => {
                Arc::new(self.as_mixed_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeMixed(_, Dimension::XY) => {
                Arc::new(self.as_large_mixed_2d().try_map_coords(map_op)?)
            }
            GeoDataType::GeometryCollection(_, Dimension::XY) => {
                Arc::new(self.as_geometry_collection_2d().try_map_coords(map_op)?)
            }
            GeoDataType::LargeGeometryCollection(_, Dimension::XY) => Arc::new(
                self.as_large_geometry_collection_2d()
                    .try_map_coords(map_op)?,
            ),
            GeoDataType::Rect(Dimension::XY) => Arc::new(self.as_rect_2d().try_map_coords(map_op)?),
            _ => return Err(GeoArrowError::IncorrectType("".into())),
        };
        Ok(result)
    }
}
