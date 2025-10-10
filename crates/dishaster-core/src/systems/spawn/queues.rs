use crate::systems::prelude::*;

/// System that spawns queue lanes for each window at level start
pub fn spawn_window_queues(
    mut commands: Commands,
    windows: Query<(Entity, &Window)>,
    canteen: Res<Canteen>,
    registry: Res<GameModelRegistryRes>,
) {
    for (window_entity, window) in windows.iter() {
        let service_template = registry.window_services.get(window.service_template);

        let mut lane_entities = vec![];
        for &offset_x in &service_template.layout.queue_x {
            let pos_x = window.location.x_min + offset_x;

            // Create the queue lane entity
            let lane_anchor = vec2(pos_x, canteen.model.windows_y - 0.5);
            let lane_entity = commands
                .spawn((
                    QueueLane {
                        owner: window_entity,
                        anchor: lane_anchor,
                        direction: Vec2::Y, // Towards the window
                    },
                    QueueLaneMembers {
                        members: vec![],
                        rear_pos: lane_anchor,
                    },
                ))
                .id();
            lane_entities.push(lane_entity);
        }

        // Now spawn the staff entity
        commands.entity(window_entity).insert(LaneOwner {
            lanes: lane_entities,
        });
    }
}
