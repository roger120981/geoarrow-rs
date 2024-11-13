use crate::array::{MultiPolygonArray, MultiPolygonBuilder};
use crate::error::Result;
use crate::io::geos::scalar::GEOSMultiPolygon;

impl<const D: usize> MultiPolygonBuilder<D> {
    pub fn from_geos(value: Vec<Option<geos::Geometry>>) -> Result<Self> {
        // TODO: don't use new_unchecked
        let geos_objects: Vec<Option<GEOSMultiPolygon>> = value
            .into_iter()
            .map(|geom| geom.map(GEOSMultiPolygon::new_unchecked))
            .collect();
        Ok(geos_objects.into())
    }
}

impl<const D: usize> MultiPolygonArray<D> {
    pub fn from_geos(value: Vec<Option<geos::Geometry>>) -> Result<Self> {
        let mutable_arr = MultiPolygonBuilder::from_geos(value)?;
        Ok(mutable_arr.into())
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use super::*;
    use crate::test::multipolygon::mp_array;
    use crate::trait_::{ArrayAccessor, NativeScalar};

    #[test]
    fn geos_round_trip() {
        let arr = mp_array();
        let geos_geoms: Vec<Option<geos::Geometry>> = arr
            .iter()
            .map(|opt_x| opt_x.map(|x| x.to_geos().unwrap()))
            .collect();
        let round_trip = MultiPolygonArray::<2>::from_geos(geos_geoms).unwrap();
        assert_eq!(arr, round_trip);
    }
}