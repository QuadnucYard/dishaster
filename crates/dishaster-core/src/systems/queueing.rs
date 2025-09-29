use std::{cmp::Ordering, collections::HashMap};

use crate::{components::*, constants::*, prelude::*, resources::*};

struct WindowQueueLayout {
    front_anchor: Vec2,
}

/// Update queue slot assignments and movement targets for diners at service windows.
pub fn update_window_queues(
    mut commands: Commands,
    canteen: Res<Canteen>,
    registry: Res<GameModelRegistryRes>,
    windows: Query<(Entity, &Window)>,
    mut diners: Query<(
        Entity,
        &mut Movement,
        &mut QueueParticipant,
        &DinerState,
        &DinerTargets,
    )>,
) {
    // Pre-compute queue lane layouts per window for quick lookup.
    let mut layout_cache: HashMap<Entity, WindowQueueLayout> = HashMap::new();
    for (window_entity, window) in windows.iter() {
        let service = registry.window_services.get(window.service_template);
        let queue_positions_world: Vec<f32> = if service.layout.queue_x.is_empty() {
            vec![window.position.center()]
        } else {
            service
                .layout
                .queue_x
                .iter()
                .map(|offset| window.position.x_min + *offset)
                .collect()
        };
        let anchor_x =
            queue_positions_world.iter().copied().sum::<f32>() / queue_positions_world.len() as f32;
        let front_y =
            (canteen.model.windows_y - WINDOW_APPROACH_OFFSET).clamp(0.0, canteen.model.height);
        layout_cache.insert(
            window_entity,
            WindowQueueLayout {
                front_anchor: Vec2::new(anchor_x.clamp(0.0, canteen.model.width), front_y),
            },
        );
    }

    if layout_cache.is_empty() {
        return;
    }

    // Group queue participants per window ordered by join time.
    let mut window_buckets: HashMap<Entity, Vec<(Entity, f64)>> = HashMap::new();
    let mut stale_participants: Vec<Entity> = Vec::new();

    for (entity, _movement, mut queue_entry, state, targets) in diners.iter_mut() {
        if !matches!(
            state.current,
            DinerStateType::MovingToWindow | DinerStateType::Queueing | DinerStateType::BeingServed
        ) {
            stale_participants.push(entity);
            continue;
        }

        if let Some(chosen) = targets.chosen_window {
            queue_entry.window = chosen;
        }

        if layout_cache.contains_key(&queue_entry.window) {
            window_buckets
                .entry(queue_entry.window)
                .or_default()
                .push((entity, queue_entry.joined_at));
        } else {
            stale_participants.push(entity);
        }
    }

    for members in window_buckets.values_mut() {
        members.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.0.index().cmp(&b.0.index()))
        });
    }

    let mut slot_lookup: HashMap<Entity, usize> = HashMap::new();
    for members in window_buckets.values() {
        for (idx, (entity, _)) in members.iter().enumerate() {
            slot_lookup.insert(*entity, idx);
        }
    }

    for (entity, mut movement, mut queue_entry, _state, _targets) in diners.iter_mut() {
        let Some(slot_index) = slot_lookup.get(&entity) else {
            continue;
        };

        let Some(layout) = layout_cache.get(&queue_entry.window) else {
            stale_participants.push(entity);
            continue;
        };

        queue_entry.slot_index = *slot_index;
        let depth = *slot_index as f32;
        let mut target = Vec2::new(
            layout.front_anchor.x,
            (layout.front_anchor.y - depth * QUEUE_SPACING).max(0.0),
        );
        target.x = target.x.clamp(0.0, canteen.model.width);
        target.y = target.y.clamp(0.0, canteen.model.height);

        if movement.target_pos.distance_squared(target) > 0.01 {
            movement.target_pos = target;
            movement.path.clear();
            movement.path.push(target);
            movement.next_waypoint = target;
        }
    }

    for entity in stale_participants {
        commands.entity(entity).remove::<QueueParticipant>();
    }
}
