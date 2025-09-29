use crate::prelude::*;

pub trait VectorExt {
    fn close_to(self, other: Self, threshold: f32) -> bool;
}

impl VectorExt for Vec2 {
    fn close_to(self, other: Self, threshold: f32) -> bool {
        (self - other).length_squared() < threshold * threshold
    }
}

impl VectorExt for Vec3 {
    fn close_to(self, other: Self, threshold: f32) -> bool {
        (self - other).length_squared() < threshold * threshold
    }
}
