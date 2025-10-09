use ordered_float::NotNan;
use rustc_hash::FxHashMap;

use super::prelude::*;

/// System that updates entities with QueueIntent to move them to the end of the specified queue lane.
pub fn update_queue_intents(
    mut commands: Commands,
    preparer_query: Query<(Entity, &mut Movement, &QueueIntent)>,
    lane_query: Query<&QueueLaneMembers>,
    mut rng: ResMut<GameRng>,
) {
    for (entity, mut movement, intent) in preparer_query {
        if movement.has_path() {
            continue;
        }
        let Ok(members) = lane_query.get(intent.lane) else {
            log::warn!(
                target: "queue",
                "QueueIntent entity {:?} references invalid lane {:?}",
                entity,
                intent.lane
            );
            continue;
        };
        // Determine the target position at the rear of the queue
        let offset = vec2(rng.random_range(-0.05..0.05), rng.random_range(-0.05..0.05)); // Slight random offset to avoid perfect overlap
        let target_position = members.rear_pos + offset;

        if movement.pos.close_to(target_position, 0.2) {
            log::debug!(
                target: "queue",
                "Entity {:?} reached end of queue lane {:?} at position {:?}",
                entity,
                intent.lane,
                target_position
            );
            movement.stop();
            // Reached the end of the queue, become a QueueMember
            commands.entity(entity).remove::<QueueIntent>();
            commands
                .entity(entity)
                .insert(QueueMember::new(intent.lane));
        } else {
            // Request a path to the target position
            log::debug!(
                target: "queue",
                "Entity {:?} moving to queue lane {:?} at position {:?}",
                entity,
                intent.lane,
                target_position
            );
            movement.request_path(target_position);
        }
    }
}

/// System that updates the positions of all QueueMember entities to maintain the queue formation.
pub fn update_queue_members(
    mut commands: Commands,
    mut member_query: Query<(Entity, &mut Movement, &mut QueueMember)>,
    mut lane_query: Query<(Entity, &QueueLane, &mut QueueLaneMembers)>,
    mut rng: ResMut<GameRng>,
) {
    // Collect members by lane
    let mut lane_to_members: FxHashMap<Entity, Vec<(Entity, Vec2, f32)>> = FxHashMap::default();
    for (entity, movement, member) in member_query.iter() {
        let Ok((_, _, members)) = lane_query.get(member.lane) else {
            log::warn!(
                target: "queue",
                "QueueMember {} references invalid lane {}",
                entity,
                member.lane
            );
            // Invalid lane reference, remove the QueueMember component
            commands.entity(entity).remove::<QueueMember>();
            continue;
        };

        let distance = -movement.pos.dot(members.rear_pos); // Negative dot product for distance along lane direction
        lane_to_members
            .entry(member.lane)
            .or_default()
            .push((entity, movement.pos, distance));
    }

    // Sort and assign rankings within each lane
    for (lane_entity, mut members) in lane_to_members {
        let Ok((_, lane, mut lane_members)) = lane_query.get_mut(lane_entity) else {
            log::warn!(target: "queue", "Invalid lane entity {}", lane_entity);
            continue;
        };
        // Sort by distance to rear position (closest first)
        members.sort_by_key(|&(_, _, dist)| NotNan::new(dist).unwrap());

        // Update each member's ranking and rebuild the members list
        lane_members.members.clear();
        for (rank, &(entity, _, _)) in members.iter().enumerate() {
            if let Ok((_, _, mut member)) = member_query.get_mut(entity) {
                member.ranking = rank;
                lane_members.members.push(entity);
            }
        }

        // Update rear position for the next member
        lane_members.rear_pos = if let Some(last_member_entity) = lane_members.members.last()
            && let Ok((_, last_movement, _)) = member_query.get_mut(*last_member_entity)
        {
            last_movement.pos - lane.direction * 0.5 // Assuming 0.5 units spacing
        } else {
            lane.anchor
        };

        // Maintain the queue formation
        for (i, member_entity) in lane_members.members.iter().enumerate() {
            let target_pos = if i == 0 {
                // First in line goes to the anchor position
                lane.anchor
            } else {
                // Subsequent members position behind the previous one
                members[i - 1].1 - lane.direction * 0.5 // Assuming 0.5 units spacing
            };
            let offset = vec2(rng.random_range(-0.1..0.1), rng.random_range(-0.1..0.1)); // Slight random offset to avoid perfect overlap
            let target_pos = target_pos + offset;
            let Ok((_, mut movement, _)) = member_query.get_mut(*member_entity) else {
                continue;
            };
            if !movement.pos.close_to(target_pos, 0.3) && !movement.has_path() {
                movement.request_path(target_pos);
                log::debug!(
                    target: "queue",
                    "Entity {} in lane {} moving to position {:.2}",
                    member_entity,
                    lane_entity,
                    target_pos
                );
            }
        }
    }
}
