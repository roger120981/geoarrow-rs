use crate::array::offset_builder::OffsetsBuilder;
use crate::array::{PointArray, WKBArray};
use crate::trait_::ArrayAccessor;
use crate::ArrayBase;
use arrow_array::{GenericBinaryArray, OffsetSizeTrait};
use arrow_buffer::Buffer;
use std::io::Cursor;
use wkb::writer::{point_wkb_size, write_point};
use wkb::Endianness;

impl<O: OffsetSizeTrait, const D: usize> From<&PointArray<D>> for WKBArray<O> {
    fn from(value: &PointArray<D>) -> Self {
        let non_null_count = value
            .nulls()
            .map_or(value.len(), |validity| value.len() - validity.null_count());

        let validity = value.nulls().cloned();
        // only allocate space for a WKBPoint for non-null items
        let values_len = non_null_count * point_wkb_size(geo_traits::Dimensions::Unknown(D));
        let mut offsets: OffsetsBuilder<O> = OffsetsBuilder::with_capacity(value.len());

        let values = {
            let values = Vec::with_capacity(values_len);
            let mut writer = Cursor::new(values);

            for maybe_geom in value.iter() {
                if let Some(geom) = maybe_geom {
                    write_point(&mut writer, &geom, Endianness::LittleEndian).unwrap();
                    offsets
                        .try_push_usize(point_wkb_size(geo_traits::Dimensions::Unknown(D)))
                        .unwrap();
                } else {
                    offsets.extend_constant(1);
                }
            }

            writer.into_inner()
        };

        let binary_arr =
            GenericBinaryArray::new(offsets.into(), Buffer::from_vec(values), validity);
        WKBArray::new(binary_arr, value.metadata())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::point::{p0, p1, p2};

    #[test]
    fn round_trip() {
        // TODO: test with nulls
        let orig_arr: PointArray<2> = vec![Some(p0()), Some(p1()), Some(p2())].into();
        let wkb_arr: WKBArray<i32> = (&orig_arr).into();
        let new_arr: PointArray<2> = wkb_arr.try_into().unwrap();

        assert_eq!(orig_arr, new_arr);
    }

    #[test]
    fn round_trip_with_null() {
        let orig_arr: PointArray<2> = vec![Some(p0()), None, Some(p1()), None, Some(p2())].into();
        let wkb_arr: WKBArray<i32> = (&orig_arr).into();
        let new_arr: PointArray<2> = wkb_arr.try_into().unwrap();

        assert_eq!(orig_arr, new_arr);
    }
}