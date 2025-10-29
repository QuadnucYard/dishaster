use dishaster_interface::snapshots::*;
use dishaster_navigation::NavigationGrid;
use rustc_hash::FxHashMap;

use crate::{components::*, prelude::*, sim::Simulation};

impl Simulation {
    pub(crate) fn snapshot_debug(&mut self) -> DebugSnapshots {
        DebugSnapshots {
            movement: self.snapshot_movement(),
            queues: self.snapshot_queue(),
            collision: self.snapshot_collision(),
            crowd: self.snapshot_crowd(),
            diners: self.snapshot_diners(),
        }
    }

    fn snapshot_movement(&mut self) -> Option<Vec<MovementDebugSnapshot>> {
        if !self.debug_flags.movement {
            return None;
        }

        let mut movement_query = self.world.query::<(Entity, &Movement)>();
        Some(
            movement_query
                .iter(&self.world)
                .map(|(entity, movement)| MovementDebugSnapshot {
                    core_id: entity.into(),
                    position: movement.pos,
                    velocity: movement.velocity,
                    path: movement.path.waypoints.clone(),
                })
                .collect(),
        )
    }

    fn snapshot_queue(&mut self) -> Option<Vec<QueueLaneDebugSnapshot>> {
        if !self.debug_flags.queues {
            return None;
        }

        let mut intents_by_lane: FxHashMap<Entity, Vec<_>> = FxHashMap::default();
        let mut intent_query = self.world.query::<(Entity, &QueueIntent, &Movement)>();
        for (entity, intent, movement) in intent_query.iter(&self.world) {
            intents_by_lane
                .entry(intent.lane)
                .or_default()
                .push(QueueIntentDebugSnapshot {
                    core_id: entity.into(),
                    position: movement.pos,
                });
        }

        let mut lane_query = self
            .world
            .query::<(Entity, &QueueLane, &QueueLaneMembers)>();
        let lanes = lane_query
            .iter(&self.world)
            .map(|(lane_entity, lane, members)| {
                let mut member_snapshots = Vec::with_capacity(members.members.len());
                for &member_entity in members.members.iter() {
                    let Some(movement) = self.world.get::<Movement>(member_entity) else {
                        continue;
                    };
                    member_snapshots.push(QueueMemberDebugSnapshot {
                        core_id: member_entity.into(),
                        position: movement.pos,
                    });
                }

                QueueLaneDebugSnapshot {
                    lane_id: lane_entity.into(),
                    anchor: lane.anchor,
                    direction: lane.direction,
                    rear_pos: members.rear_pos,
                    members: member_snapshots,
                    intents: intents_by_lane.remove(&lane_entity).unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();

        Some(lanes)
    }

    fn snapshot_collision(&mut self) -> Option<CollisionGridDebugSnapshot> {
        None
        // if !self.debug_flags.nav_grid {
        //     return None;
        // }

        // let grid = self.world.resource::<CollisionGridRes>();
        // let cells = grid.debug_cells();
        // if cells.is_empty() {
        //     return None;
        // }

        // let cell_size = grid.cell_size();
        // let size = Vec2::splat(cell_size);
        // Some(CollisionGridDebugSnapshot {
        //     cell_size,
        //     cells: cells
        //         .into_iter()
        //         .map(|(coord, count)| {
        //             let center = grid.tile_to_world(coord);
        //             CollisionCellDebugSnapshot {
        //                 coord,
        //                 center,
        //                 size,
        //                 occupancy: count as u32,
        //             }
        //         })
        //         .collect(),
        // })
    }

    fn snapshot_crowd(&mut self) -> Option<CrowdFieldDebugSnapshot> {
        if !self.debug_flags.crowd_field {
            return None;
        }

        let field = &self.world.resource::<ResWrapper<NavigationGrid>>().crowd;

        let cell_size = field.cell_size();
        let dimensions = field.tile_dimensions();
        let tiles = field.costs().flatten().clone();
        Some(CrowdFieldDebugSnapshot {
            cell_size,
            origin: Default::default(),
            dimensions,
            costs: tiles,
        })
    }

    fn snapshot_diners(&mut self) -> Option<Vec<DinerDebugSnapshot>> {
        if !self.debug_flags.diners {
            return None;
        }

        let mut diner_query = self.world.query::<(Entity, &DinerGoalState)>();
        Some(
            diner_query
                .iter(&self.world)
                .map(|(entity, goals)| DinerDebugSnapshot {
                    core_id: entity.into(),
                    goal_str: eco_format!("{:?}", goals.current()),
                })
                .collect(),
        )
    }
}
