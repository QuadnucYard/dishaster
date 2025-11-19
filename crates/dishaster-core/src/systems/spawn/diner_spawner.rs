use std::sync::LazyLock;

use crate::systems::prelude::*;

/// System that manages diner spawning based on pre-scheduled arrival times
pub fn update_diner_spawner(
    mut commands: Commands,
    time: Res<Time>,
    day_status: Res<DayStatus>,
    mut schedule: ResMut<DailyDinerSchedule>,
    canteen: Res<Canteen>,
    mut rng: ResMut<SpawnerRng>,
) {
    // Use relative time for arrival time checks
    let current_time = time.world_time as f32 - day_status.start_time;

    // Spawn all diners whose arrival time has passed
    while let Some(scheduled) = schedule.next_diner_if_ready(current_time) {
        spawn_scheduled_diner(scheduled, &mut commands, &canteen, &mut rng);
    }
}

/// Spawn a scheduled diner entity at the canteen entrance
fn spawn_scheduled_diner(
    scheduled: ScheduledDiner,
    commands: &mut Commands,
    canteen: &Canteen,
    rng: &mut Prng,
) {
    let diner_id = scheduled.id;

    // Sample spawn position from entrance
    let pos = {
        let entrance = canteen
            .model
            .entrances
            .choose(rng)
            .expect("canteen must have at least one entrance");
        let x = (entrance.x_min + entrance.x_max) / 2.0; // Center of entrance
        Vec2::new(x, canteen.model.entrances_y + 0.5)
    };

    log::info!(
        target: "diner",
        "spawn: id={} pos={:.2}",
        diner_id,
        pos,
    );

    let display_res = PrefabRef::new("diners/sample_diner");

    let wrapper = commands.spawn((
        AgentTag,
        DinerBundle {
            diner: Diner { id: diner_id },

            state: DinerState {
                meal_budget: scheduled.meal_budget,
                ..Default::default()
            },
            goal: DinerGoalState::default(),
            targets: DinerTargets::default(),

            personality: scheduled.personality.into_comp(),
            dining_profile: scheduled.dining_profile.into_comp(),
            psych_state: scheduled.psych_state.into_comp(),
            ltm: scheduled.long_term_memory.into_comp(),
            stm: DinerShortTermMemory::default(),
            appearance: scheduled.appearance.into_comp(),

            movement: Movement {
                pos,
                radius: 0.15,
                impatience: 0.7,
                avoidance_responsibility: 2.0,
                ..Default::default()
            },
        },
        DisplayState {
            name: Some(eco_format!("Diner_{}", diner_id)),
            ..Default::default()
        },
        Transform {
            position: pos.extend(0.0),
            ..Default::default()
        },
        EntityRng::new(diner_id as u64),
    ));
    let wrapper_entity = wrapper.id();

    let _body = commands.spawn((
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

    static FEEDBACK_PROTO: LazyLock<PrefabRef> =
        LazyLock::new(|| PrefabRef::new("feedback_balloon"));
    static DEBUG_PROTO: LazyLock<PrefabRef> = LazyLock::new(|| PrefabRef::new("agent_debug"));

    let _feedback = commands.spawn((
        DisplayState {
            proto: FEEDBACK_PROTO.clone(),
            name: Some("Feedback".into()),
        },
        Transform {
            position: vec3(0.0, 0.0, 1.7),
            parent: Some(wrapper_entity),
            ..Default::default()
        },
        ChildOf(wrapper_entity),
    ));

    let _debug = commands.spawn((
        DisplayState {
            proto: DEBUG_PROTO.clone(),
            name: Some("Debug".into()),
        },
        Transform {
            position: vec3(0.0, 0.0, 1.8),
            parent: Some(wrapper_entity),
            ..Default::default()
        },
        ChildOf(wrapper_entity),
    ));
}
