use core::f64;

use geo_traits::{CoordTrait, PointTrait};
use geoarrow_schema::{CoordType, Dimension};

use crate::array::{CoordBuffer, InterleavedCoordBufferBuilder, SeparatedCoordBufferBuilder};
use crate::error::Result;

/// The GeoArrow equivalent to `Vec<Coord>`: a mutable collection of coordinates.
///
/// Converting an [`CoordBufferBuilder`] into a [`CoordBuffer`] is `O(1)`.
#[derive(Debug, Clone)]
pub enum CoordBufferBuilder {
    /// Interleaved coordinates
    Interleaved(InterleavedCoordBufferBuilder),
    /// Separated coordinates
    Separated(SeparatedCoordBufferBuilder),
}

impl CoordBufferBuilder {
    /// Initialize a buffer of a given length with all coordinates set to 0.0
    pub fn initialize(len: usize, interleaved: bool, dim: Dimension) -> Self {
        match interleaved {
            true => {
                CoordBufferBuilder::Interleaved(InterleavedCoordBufferBuilder::initialize(len, dim))
            }
            false => {
                CoordBufferBuilder::Separated(SeparatedCoordBufferBuilder::initialize(len, dim))
            }
        }
    }

    /// Reserves capacity for at least `additional` more coordinates to be inserted
    /// in the given `Vec<T>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    pub fn reserve(&mut self, additional: usize) {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.reserve(additional),
            CoordBufferBuilder::Separated(cb) => cb.reserve(additional),
        }
    }

    /// Reserves the minimum capacity for at least `additional` more coordinates.
    ///
    /// Unlike [`reserve`], this will not deliberately over-allocate to speculatively avoid
    /// frequent allocations. After calling `reserve_exact`, capacity will be greater than or equal
    /// to `self.len() + additional`. Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer [`reserve`] if future insertions are expected.
    ///
    /// [`reserve`]: Self::reserve
    pub fn reserve_exact(&mut self, additional: usize) {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.reserve_exact(additional),
            CoordBufferBuilder::Separated(cb) => cb.reserve_exact(additional),
        }
    }

    /// Returns the total number of coordinates the vector can hold without reallocating.
    pub fn capacity(&self) -> usize {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.capacity(),
            CoordBufferBuilder::Separated(cb) => cb.capacity(),
        }
    }

    /// The number of coordinates
    pub fn len(&self) -> usize {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.len(),
            CoordBufferBuilder::Separated(cb) => cb.len(),
        }
    }

    /// Whether the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The underlying coordinate type
    pub fn coord_type(&self) -> CoordType {
        match self {
            CoordBufferBuilder::Interleaved(_) => CoordType::Interleaved,
            CoordBufferBuilder::Separated(_) => CoordType::Separated,
        }
    }

    /// Push a new coord onto the end of this coordinate buffer
    ///
    /// ## Panics
    ///
    /// - If the added coordinate does not have the same dimension as the coordinate buffer.
    pub fn push_coord(&mut self, coord: &impl CoordTrait<T = f64>) {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.push_coord(coord),
            CoordBufferBuilder::Separated(cb) => cb.push_coord(coord),
        }
    }

    /// Push a new coord onto the end of this coordinate buffer
    ///
    /// ## Errors
    ///
    /// - If the added coordinate does not have the same dimension as the coordinate buffer.
    pub fn try_push_coord(&mut self, coord: &impl CoordTrait<T = f64>) -> Result<()> {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.try_push_coord(coord),
            CoordBufferBuilder::Separated(cb) => cb.try_push_coord(coord),
        }
    }

    /// Push a valid coordinate with NaN values
    ///
    /// Used in the case of point and rect arrays, where a `null` array value still needs to have
    /// space allocated for it.
    pub fn push_nan_coord(&mut self) {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.push_nan_coord(),
            CoordBufferBuilder::Separated(cb) => cb.push_nan_coord(),
        }
    }

    /// Push a new point onto the end of this coordinate buffer
    ///
    /// ## Panics
    ///
    /// - If the added point does not have the same dimension as the coordinate buffer.
    pub fn push_point(&mut self, point: &impl PointTrait<T = f64>) {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.push_point(point),
            CoordBufferBuilder::Separated(cb) => cb.push_point(point),
        }
    }

    /// Push a new point onto the end of this coordinate buffer
    ///
    /// ## Errors
    ///
    /// - If the added point does not have the same dimension as the coordinate buffer.
    pub fn try_push_point(&mut self, point: &impl PointTrait<T = f64>) -> Result<()> {
        match self {
            CoordBufferBuilder::Interleaved(cb) => cb.try_push_point(point),
            CoordBufferBuilder::Separated(cb) => cb.try_push_point(point),
        }
    }
}

impl From<CoordBufferBuilder> for CoordBuffer {
    fn from(value: CoordBufferBuilder) -> Self {
        match value {
            CoordBufferBuilder::Interleaved(cb) => CoordBuffer::Interleaved(cb.into()),
            CoordBufferBuilder::Separated(cb) => CoordBuffer::Separated(cb.into()),
        }
    }
}
