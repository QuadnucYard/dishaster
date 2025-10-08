use dishrupt_core::{
    asset::PrefabReference,
    display::{DisplayState, Transform},
};

use crate::systems::prelude::*;

/// System that spawns all serving staff at level start, associated with their respective windows
pub fn spawn_serving_staffs(
    mut commands: Commands,
    windows: Query<(Entity, &Window)>,
    canteen: Res<Canteen>,
    registry: Res<GameModelRegistryRes>,
    display_root: Res<DisplayRoot>,
    mut staff_registry: ResMut<ServingStaffRegistry>,
) {
    for (window_entity, window) in windows.iter() {
        // Spawn serving staff associated with this window.
        let staff_y =
            (canteen.model.windows_y + WINDOW_STAFF_OFFSET).clamp(0.0, canteen.model.height);
        let service_template = registry.window_services.get(window.service_template);

        for (queue_index, &pos_x) in service_template.layout.queue_x.iter().enumerate() {
            let pos_x = window.position.x_min + pos_x;
            let staff_pos = vec2(pos_x, staff_y);

            let display_res = PrefabReference::new("staffs/sample_staff"); // placeholder
            let staff_cmd = commands.spawn((
                AgentTag,
                ServingStaffBundle {
                    staff: ServingStaff {
                        window: window_entity,
                    },
                    state: ServingStaffState::default(),
                    movement: Movement {
                        pos: staff_pos,
                        walking_speed: STAFF_WALK_SPEED,
                        radius: STAFF_COLLISION_RADIUS,
                        ..Default::default()
                    },
                },
                DisplayState {
                    name: Some(eco_format!(
                        "Staff_{}_{}",
                        window.config.slot_index,
                        queue_index
                    )),
                    ..Default::default()
                },
                Transform {
                    position: staff_pos.extend(0.0),
                    parent: Some(display_root.0),
                    ..Default::default()
                },
            ));
            let staff_entity = staff_cmd.id();
            let _body_cmd = commands.spawn((
                DisplayState {
                    proto: display_res,
                    ..Default::default()
                },
                Transform {
                    position: Vec3::ZERO,
                    parent: Some(staff_entity),
                    ..Default::default()
                },
                ChildOf(staff_entity),
            ));

            let _feedback_cmd = commands.spawn((
                DisplayState {
                    proto: PrefabReference::new("feedback_balloon"),
                    name: Some("Feedback".into()),
                    ..Default::default()
                },
                Transform {
                    position: vec3(0.0, 0.0, 1.8),
                    parent: Some(staff_entity),
                    ..Default::default()
                },
                ChildOf(staff_entity),
            ));

            staff_registry.register(window_entity, staff_entity);
        }
    }
}
