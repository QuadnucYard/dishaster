use crate::{events::TrialStart, systems::prelude::*};

pub fn crab_trial_system(
    mut commands: Commands,
    mut crab: Option<ResMut<CrabTurmoil>>,
    diner_query: Query<(Entity, &DinerGoalState)>,
    mut rng: ResMut<CrabRng>,
    time: Res<Time>,
) {
    let Some(crab) = &mut crab else {
        return;
    };
    if crab.triggered_diners.len() as u32 >= crab.trigger_limit {
        return;
    }
    let dt = time.tick_duration;
    for (entity, goal) in diner_query.iter() {
        if goal.is(DinerGoal::Observe)
            && rng.random_bool_dt(crab.probability as f64, dt)
            && crab.triggered_diners.insert(entity)
        {
            commands.trigger(TrialStart {
                diner: entity,
                topic: Some(FeedbackTopic::Crab),
            });
        }
    }
}
