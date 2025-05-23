use crate::error::{GeoArrowError, Result};
use crate::io::geos::scalar::GEOSConstLinearRing;
use crate::io::geos::scalar::linestring::to_geos_linear_ring;
use crate::scalar::Polygon;
use geo_traits::PolygonTrait;
use geos::{Geom, GeometryTypes};

impl<'a> TryFrom<&'a Polygon<'_>> for geos::Geometry {
    type Error = geos::Error;

    fn try_from(value: &'a Polygon<'_>) -> std::result::Result<geos::Geometry, geos::Error> {
        to_geos_polygon(value)
    }
}

pub(crate) fn to_geos_polygon(
    polygon: &impl PolygonTrait<T = f64>,
) -> std::result::Result<geos::Geometry, geos::Error> {
    if let Some(exterior) = polygon.exterior() {
        let exterior = to_geos_linear_ring(&exterior)?;
        let interiors = polygon
            .interiors()
            .map(|interior| to_geos_linear_ring(&interior))
            .collect::<std::result::Result<Vec<_>, geos::Error>>()?;
        geos::Geometry::create_polygon(exterior, interiors)
    } else {
        geos::Geometry::create_empty_polygon()
    }
}

#[derive(Clone)]
pub struct GEOSPolygon(pub(crate) geos::Geometry);

impl GEOSPolygon {
    pub fn new_unchecked(geom: geos::Geometry) -> Self {
        Self(geom)
    }

    #[allow(dead_code)]
    pub fn try_new(geom: geos::Geometry) -> Result<Self> {
        if matches!(geom.geometry_type(), GeometryTypes::Polygon) {
            Ok(Self(geom))
        } else {
            Err(GeoArrowError::General(
                "Geometry type must be polygon".to_string(),
            ))
        }
    }

    // TODO: delete these
    #[allow(dead_code)]
    pub fn num_interiors(&self) -> usize {
        self.0.get_num_interior_rings().unwrap()
    }

    #[allow(dead_code)]
    pub fn exterior(&self) -> Option<GEOSConstLinearRing<'_>> {
        if self.0.is_empty().unwrap() {
            return None;
        }

        Some(GEOSConstLinearRing::new_unchecked(
            self.0.get_exterior_ring().unwrap(),
        ))
    }

    #[allow(dead_code)]
    pub fn interior(&self, i: usize) -> Option<GEOSConstLinearRing<'_>> {
        if i > self.num_interiors() {
            return None;
        }

        Some(GEOSConstLinearRing::new_unchecked(
            self.0.get_interior_ring_n(i.try_into().unwrap()).unwrap(),
        ))
    }
}

impl PolygonTrait for GEOSPolygon {
    type T = f64;
    type RingType<'a>
        = GEOSConstLinearRing<'a>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        match self.0.get_coordinate_dimension().unwrap() {
            geos::Dimensions::TwoD => geo_traits::Dimensions::Xy,
            geos::Dimensions::ThreeD => geo_traits::Dimensions::Xyz,
            geos::Dimensions::Other(other) => panic!("Other dimensions not supported {other}"),
        }
    }

    fn num_interiors(&self) -> usize {
        self.0.get_num_interior_rings().unwrap()
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        if self.0.is_empty().unwrap() {
            return None;
        }

        Some(GEOSConstLinearRing::new_unchecked(
            self.0.get_exterior_ring().unwrap(),
        ))
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        GEOSConstLinearRing::new_unchecked(
            self.0.get_interior_ring_n(i.try_into().unwrap()).unwrap(),
        )
    }
}

pub struct GEOSConstPolygon<'a>(pub(crate) geos::ConstGeometry<'a>);

impl<'a> GEOSConstPolygon<'a> {
    pub fn new_unchecked(geom: geos::ConstGeometry<'a>) -> Self {
        Self(geom)
    }

    #[allow(dead_code)]
    pub fn try_new(geom: geos::ConstGeometry<'a>) -> Result<Self> {
        if matches!(geom.geometry_type(), GeometryTypes::Polygon) {
            Ok(Self(geom))
        } else {
            Err(GeoArrowError::General(
                "Geometry type must be polygon".to_string(),
            ))
        }
    }
}

impl PolygonTrait for GEOSConstPolygon<'_> {
    type T = f64;
    type RingType<'c>
        = GEOSConstLinearRing<'c>
    where
        Self: 'c;

    fn dim(&self) -> geo_traits::Dimensions {
        match self.0.get_coordinate_dimension().unwrap() {
            geos::Dimensions::TwoD => geo_traits::Dimensions::Xy,
            geos::Dimensions::ThreeD => geo_traits::Dimensions::Xyz,
            geos::Dimensions::Other(other) => panic!("Other dimensions not supported {other}"),
        }
    }

    fn num_interiors(&self) -> usize {
        self.0.get_num_interior_rings().unwrap()
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        if self.0.is_empty().unwrap() {
            return None;
        }

        Some(GEOSConstLinearRing::new_unchecked(
            self.0.get_exterior_ring().unwrap(),
        ))
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        GEOSConstLinearRing::new_unchecked(
            self.0.get_interior_ring_n(i.try_into().unwrap()).unwrap(),
        )
    }
}
