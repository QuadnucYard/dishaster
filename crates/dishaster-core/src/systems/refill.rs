use super::prelude::*;

/// Spawn refill staff for the requested dispenser
pub fn handle_refill_request(
    mut commands: Commands,
    mut messages: MessageReader<RefillDispenser>,
    mut dispenser_query: Query<&mut Dispenser>,
    canteen: Res<Canteen>,
    display_root: Res<DisplayRoot>,
) {
    for message in messages.read() {
        let dispenser_entity = message.0;
        let Ok(mut dispenser) = dispenser_query.get_mut(dispenser_entity) else {
            log::warn!(
                target: "refill",
                "Dispenser not found: {dispenser_entity:?}"
            );
            continue;
        };

        if dispenser.refill_pending {
            log::debug!(
                target: "refill",
                "Refill already pending for dispenser: {dispenser_entity:?}"
            );
            continue;
        };

        // Mark as pending
        dispenser.refill_pending = true;

        // Spawn refill staff at corner of canteen (entrance area)
        let spawn_pos = vec2(1., canteen.model.entrances_y + 1.);

        let display_res = PrefabReference::new("staffs/sample_staff");
        let wrapper = commands.spawn((
            AgentTag,
            RefillStaffBundle {
                staff: RefillStaff {
                    target_dispenser: dispenser_entity,
                    spawn_pos,
                },
                state: RefillStaffState {
                    status: RefillStaffStatus::MovingToDispenser,
                    activity_timer: 0.0,
                },
                movement: Movement {
                    pos: spawn_pos,
                    walking_speed: STAFF_WALK_SPEED,
                    radius: STAFF_COLLISION_RADIUS,
                    ..Default::default()
                },
            },
            DisplayState {
                name: Some(eco_format!("RefillStaff")),
                ..Default::default()
            },
            Transform {
                position: spawn_pos.extend(0.0),
                parent: Some(display_root.0),
                ..Default::default()
            },
        ));
        let wrapper_entity = wrapper.id();

        // Spawn visual body
        commands.spawn((
            DisplayState {
                proto: display_res,
                name: Some("Body".into()),
            },
            Transform {
                position: Vec3::ZERO,
                parent: Some(wrapper_entity),
                ..Default::default()
            },
            ChildOf(wrapper_entity),
        ));

        log::info!(
            target: "refill",
            "Spawned refill staff for dispenser: {dispenser_entity:?} at {spawn_pos}"
        );
    }
}

/// Handle refill staff AI: navigate to dispenser, refill, and return.
pub fn handle_refill_staff(
    mut commands: Commands,
    mut staff_query: Query<(Entity, &RefillStaff, &mut RefillStaffState, &mut Movement)>,
    mut dispenser_query: Query<&mut Dispenser>,
    registry: Res<GameModelRegistryRes>,
    time: Res<Time>,
) {
    let delta = time.tick_duration as f32;
    for (entity, staff, mut state, mut movement) in staff_query.iter_mut() {
        match state.status {
            RefillStaffStatus::MovingToDispenser => {
                // Request path to dispenser if not already moving
                if !movement.has_path() && !movement.target_reached {
                    if let Ok(dispenser) = dispenser_query.get(staff.target_dispenser) {
                        movement.request_path_to_rect(dispenser.reception_area);
                        log::debug!(
                            target: "refill_staff",
                            "staff={entity:?} moving to dispenser={:?}",
                            staff.target_dispenser
                        );
                    } else {
                        // Dispenser doesn't exist, despawn staff
                        log::warn!(
                            target: "refill_staff",
                            "staff={entity:?} target dispenser not found, despawning"
                        );
                        commands.entity(entity).despawn();
                    }
                    continue;
                }

                // Check if reached
                if movement.target_reached {
                    state.status = RefillStaffStatus::Refilling;
                    state.activity_timer = 1.5; // 1.5 seconds refill time
                    log::debug!(
                        target: "refill_staff",
                        "staff={entity:?} started refilling"
                    );
                }
            }

            RefillStaffStatus::Refilling => {
                // Count down timer
                state.activity_timer -= delta;

                if state.activity_timer <= 0.0 {
                    // Refill complete - restore stock to full capacity
                    if let Ok(mut dispenser) = dispenser_query.get_mut(staff.target_dispenser) {
                        let model = registry.dispensers.get(dispenser.model);
                        dispenser.current_stock = model.capacity;
                        dispenser.refill_pending = false;
                        log::info!(
                            target: "refill_staff",
                            "staff={entity:?} refilled dispenser={:?} to capacity={}",
                            staff.target_dispenser,
                            model.capacity
                        );
                    }

                    // Start returning
                    state.status = RefillStaffStatus::Returning;

                    movement.request_path(staff.spawn_pos);
                    log::debug!(
                        target: "refill_staff",
                        "staff={entity:?} returning to spawn pos={:?}",
                        staff.spawn_pos
                    );
                }
            }

            RefillStaffStatus::Returning => {
                // Check if reached spawn point
                if movement.target_reached {
                    log::debug!(
                        target: "refill_staff",
                        "staff={entity:?} reached spawn point, despawning"
                    );
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
