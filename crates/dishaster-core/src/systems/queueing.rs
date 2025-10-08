use std::cmp::Ordering;

use rustc_hash::FxHashMap;

use super::prelude::*;

struct WindowQueueLayout {
    lanes: Vec<Vec2>,
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
    let mut layout_cache = FxHashMap::default();
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
        let front_y =
            (canteen.model.windows_y - WINDOW_APPROACH_OFFSET).clamp(0.0, canteen.model.height);
        let lanes = queue_positions_world
            .into_iter()
            .map(|x| Vec2::new(x.clamp(0.0, canteen.model.width), front_y))
            .collect();
        layout_cache.insert(window_entity, WindowQueueLayout { lanes });
    }

    if layout_cache.is_empty() {
        return;
    }

    // Group queue participants per window ordered by join time.
    let mut window_buckets: FxHashMap<Entity, Vec<(Entity, f64, Option<usize>)>> =
        FxHashMap::default();
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
            window_buckets.entry(queue_entry.window).or_default().push((
                entity,
                queue_entry.joined_at,
                queue_entry.lane_index,
            ));
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

    let mut slot_lookup = FxHashMap::default();
    for members in window_buckets.values() {
        for (idx, (entity, _, _)) in members.iter().enumerate() {
            slot_lookup.insert(*entity, idx);
        }
    }

    let mut lane_lookup = FxHashMap::default();
    for (window, members) in &window_buckets {
        let Some(layout) = layout_cache.get(window) else {
            continue;
        };
        let lane_count = layout.lanes.len().max(1);
        let mut lane_loads = vec![0usize; lane_count];
        for (entity, _, prev_lane) in members.iter() {
            let previous = prev_lane.and_then(|lane| (lane < lane_count).then_some(lane));
            let lane = previous.unwrap_or_else(|| {
                lane_loads
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, count)| *count)
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            });
            lane_loads[lane] += 1;
            lane_lookup.insert(*entity, lane);
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

        let lane_index = lane_lookup
            .get(&entity)
            .copied()
            .unwrap_or(0)
            .min(layout.lanes.len().saturating_sub(1));
        queue_entry.slot_index = *slot_index;
        queue_entry.lane_index = Some(lane_index);
        let depth = *slot_index as f32;
        let anchor = layout
            .lanes
            .get(lane_index)
            .copied()
            .or_else(|| layout.lanes.first().copied())
            .unwrap_or(Vec2::ZERO);
        let mut target = Vec2::new(anchor.x, (anchor.y - depth * QUEUE_SPACING).max(0.0));
        target.x = target.x.clamp(0.0, canteen.model.width);
        target.y = target.y.clamp(0.0, canteen.model.height);

        if !movement.pos.close_to(target, 0.2) {
            movement.request_path(target);
        }
    }

    for entity in stale_participants {
        commands.entity(entity).remove::<QueueParticipant>();
    }
}
