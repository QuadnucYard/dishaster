use super::prelude::*;

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

    /// Convert the size to a 2D vector representation
    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

/// Horizontal range defined by minimum and maximum X coordinates
///
/// Used for defining entrance areas, window positions, and other
/// linear features along the X axis. Provides containment testing
/// and geometric calculations.
#[derive(Debug, Clone, Copy, Deserialize)]
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

/// Horizontal line segment defined by X range and Y coordinate
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct XSegment {
    /// Minimum X coordinate of the range
    pub x_min: Meters,
    /// Maximum X coordinate of the range
    pub x_max: Meters,
    /// Y coordinate for the segment line
    pub y: Meters,
}

impl XSegment {
    /// Create a new X segment with specified bounds and Y position
    pub fn new(x_min: Meters, x_max: Meters, y: Meters) -> Self {
        Self { x_min, x_max, y }
    }

    /// Calculate the center point of this range
    pub fn center(&self) -> Vec2 {
        vec2((self.x_min + self.x_max) / 2.0, self.y)
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

/// Configuration for refill staff operations
///
/// Controls limits and behavior for staff that restock dispensers.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefillConfig {
    /// Maximum number of refill staff that can be active simultaneously
    pub max_concurrent_staff: usize,
}

impl Default for RefillConfig {
    fn default() -> Self {
        Self {
            max_concurrent_staff: 1,
        }
    }
}
