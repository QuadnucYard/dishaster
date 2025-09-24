use dishrupt_core::prelude::*;

// pub trait DisplayContext<S, T> {}

/// The context for translation vectors between simulation space and display space.
#[derive(Resource, Clone)]
pub struct DisplayContext2D {
    /// Scale factors for each axis from simulation space to display space.
    pub view_scale: Vec3,
}

impl DisplayContext2D {
    pub fn aspect_ratio(&self) -> f32 {
        self.view_scale.x / self.view_scale.y
    }

    /// Convert a position in simulation space to display space.
    /// It will add z to y .
    ///
    /// x' = x * s_x,
    /// y' = y * s_y - z * s_z.
    pub fn to_display_space(&self, pos: Vec3) -> Vec2 {
        let pos = pos * self.view_scale;
        Vec2::new(pos.x, pos.y - pos.z)
    }

    /// Convert a position in display space to simulation space.
    /// Suppose it has z=0 in the original simulation space for simplicity.
    ///
    /// x = x' / s_x,
    /// y = y' / s_y.
    pub fn to_simulation_space(&self, pos: Vec2) -> Vec2 {
        Vec2::new(pos.x / self.view_scale.x, pos.y / self.view_scale.y)
    }
}
