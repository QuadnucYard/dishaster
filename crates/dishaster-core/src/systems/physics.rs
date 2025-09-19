use crate::{components::*, prelude::*, resources::*};

/// System to update the global collision grid
pub fn update_collision_grid(
    mut collision_grid: ResMut<CollisionGridRes>,
    query: Query<(Entity, &BoxCollider)>,
) {
    collision_grid.update(&query);
}
