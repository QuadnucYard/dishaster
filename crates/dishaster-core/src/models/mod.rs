//! Game model definitions and data structures

mod canteen;
mod diner;
mod dish;
mod level;
mod movement;

pub use canteen::*;
pub use diner::*;
pub use dish::*;
pub use level::*;
pub use movement::*;
use serde::Deserialize;

mod prelude {
    pub use serde::{Deserialize, Serialize};

    pub use super::{Meters, MinMax, Seconds, Size, XRange};
    pub use crate::{
        model_registry::{HasId, ModelId},
        prelude::*,
    };
}

/// Physical distance measurement in meters
///
/// Used for all spatial calculations including object sizes,
/// positions, and movement distances within the simulation.
pub type Meters = f32;

/// Time duration measurement in seconds
///
/// Used for timing calculations, delays, and duration-based
/// game mechanics throughout the simulation.
pub type Seconds = f32;

/// 2D rectangular dimensions for game objects
///
/// Represents the physical size of objects like tables, dispensers,
/// and collision boundaries. Used for spatial calculations and
/// layout validation.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Size {
    /// Width dimension in meters
    pub width: Meters,
    /// Height dimension in meters
    pub height: Meters,
}

impl Size {
    /// Create a new size with specified dimensions
    pub fn new(width: Meters, height: Meters) -> Self {
        Self { width, height }
    }
}

/// Horizontal range defined by minimum and maximum X coordinates
///
/// Used for defining entrance areas, window positions, and other
/// linear features along the X axis. Provides containment testing
/// and geometric calculations.
#[derive(Debug, Clone, Deserialize)]
pub struct XRange {
    /// Minimum X coordinate of the range
    pub x_min: Meters,
    /// Maximum X coordinate of the range
    pub x_max: Meters,
}

impl XRange {
    /// Create a new X range with specified bounds
    pub fn new(x_min: Meters, x_max: Meters) -> Self {
        Self { x_min, x_max }
    }

    /// Check if a given X coordinate falls within this range
    pub fn contains(&self, x: Meters) -> bool {
        x >= self.x_min && x <= self.x_max
    }

    /// Calculate the center point of this range
    pub fn center(&self) -> Meters {
        (self.x_min + self.x_max) / 2.0
    }

    /// Calculate the width of this range
    pub fn width(&self) -> Meters {
        self.x_max - self.x_min
    }
}

/// Generic min-max value range for statistical distributions
///
/// Used throughout the simulation for defining probability ranges,
/// attribute bounds, and random value generation. Supports any
/// numeric type for flexible usage patterns.
#[derive(Debug, Clone, Copy)]
pub struct MinMax<T = f32> {
    /// Minimum value of the range
    pub min: T,
    /// Maximum value of the range
    pub max: T,
}

impl<T> MinMax<T> {
    /// Create a new min-max range
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<'de> Deserialize<'de> for MinMax {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;

        use serde::de::{self, MapAccess, SeqAccess, Visitor};

        struct MinMaxVisitor;

        impl<'de> Visitor<'de> for MinMaxVisitor {
            type Value = MinMax;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with 'min' and 'max' fields or a tuple of two floats")
            }

            fn visit_map<V>(self, mut map: V) -> Result<MinMax, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut min = None;
                let mut max = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "min" => min = Some(map.next_value()?),
                        "max" => max = Some(map.next_value()?),
                        _ => return Err(de::Error::unknown_field(key, &["min", "max"])),
                    }
                }
                let min = min.ok_or_else(|| de::Error::missing_field("min"))?;
                let max = max.ok_or_else(|| de::Error::missing_field("max"))?;
                Ok(MinMax { min, max })
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<MinMax, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let min = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let max = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(MinMax { min, max })
            }
        }

        deserializer.deserialize_any(MinMaxVisitor)
    }
}
