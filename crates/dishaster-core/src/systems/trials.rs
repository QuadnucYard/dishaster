use dishaster_interface::{SimEvent, event::HintCondition};
use dishaster_trial as trial;

use crate::{
    events::*,
    prelude::*,
    resources::{EventQueue, GameModelRegistryRes, Time, TrialSession},
    systems::hint::{HintEmitter, hints},
};

pub fn register_trial_systems(world: &mut World) {
    world.add_observer(on_trial_start);
    world.add_observer(on_trial_launch);
    world.add_observer(on_trial_respond);
    world.add_observer(on_trial_timeout);
    world.add_observer(on_trial_request_candidates);
    world.add_observer(on_trial_proceed);
}

fn on_trial_start(
    event: On<TrialStart>,
    mut session: ResMut<TrialSession>,
    mut events: ResMut<EventQueue>,
    time: Res<Time>,
) {
    let TrialStart { diner, topic } = *event;
    let current_time = time.current_time as f32;

    if !session.can_start(current_time) {
        // Trial cooldown or already active; emit hint and ignore
        events.emit_hint(hints::TRIAL_COOLDOWN, HintCondition::Always);
        return;
    }

    // Reset trial session for new trial
    session.start(diner.to_entity_id(), topic);

    let intro = trial::create_trial_intro(&mut session);

    events.push(SimEvent::TrialIntro(intro.into()));
}

fn on_trial_launch(
    _event: On<TrialLaunch>,
    mut session: ResMut<TrialSession>,
    registry: Res<GameModelRegistryRes>,
    mut events: ResMut<EventQueue>,
) {
    let statement = trial::create_diner_statement(&mut session, &registry.trial);

    events.push(SimEvent::TrialLeftSpeak(statement.into()));
}

/// Respond with the selected speech
fn on_trial_respond(
    event: On<TrialRespond>,
    mut commands: Commands,
    mut session: ResMut<TrialSession>,
    registry: Res<GameModelRegistryRes>,
    mut events: ResMut<EventQueue>,
) {
    let resp_id = event.0;

    let (speech, impact) = trial::trial_respond(&mut session, &registry.trial, resp_id);

    if let Some(diner_entity) = session.target_entity {
        commands.trigger(ApplyTrialImpact {
            diner: diner_entity.to_entity(),
            psych_impact: impact.psych,
            reputation_impact: impact.reputation,
        });
    }

    events.push(SimEvent::TrialRightSpeak(speech.into()));
}

/// Apply timeout penalty before ending trial
fn on_trial_timeout(
    _event: On<TrialTimeout>,
    mut commands: Commands,
    mut session: ResMut<TrialSession>,
    mut events: ResMut<EventQueue>,
    time: Res<Time>,
) {
    let impact = trial::get_trial_timeout_penalty(&session.config);

    if let Some(diner_entity) = session.target_entity {
        commands.trigger(ApplyTrialImpact {
            diner: diner_entity.to_entity(),
            psych_impact: impact.psych,
            reputation_impact: impact.reputation,
        });
    }

    session.finish(time.current_time as f32);
    events.push(SimEvent::TrialEnd { timeout: true });
}

/// Generate response candidates for this keyword
fn on_trial_request_candidates(
    event: On<TrialRequestCandidates>,
    mut session: ResMut<TrialSession>,
    registry: Res<GameModelRegistryRes>,
    mut events: ResMut<EventQueue>,
) {
    let TrialRequestCandidates {
        speech_id,
        keyword_index,
    } = *event;

    let options = trial::generate_trial_response_candidates(
        &mut session,
        &registry.trial,
        speech_id,
        keyword_index,
    );

    events.push(SimEvent::TrialResponseCandidates(options));
}

fn on_trial_proceed(
    _event: On<TrialProceed>,
    mut session: ResMut<TrialSession>,
    registry: Res<GameModelRegistryRes>,
    mut events: ResMut<EventQueue>,
    time: Res<Time>,
) {
    let should_continue = trial::trial_should_continue(&mut session, &registry.trial);

    if should_continue {
        // Generate new speech sequence on a related topic
        let statement = trial::create_diner_statement(&mut session, &registry.trial);
        events.push(SimEvent::TrialLeftSpeak(statement.into()));
    } else {
        // End of trial
        session.finish(time.current_time as f32);
        events.push(SimEvent::TrialEnd { timeout: false });
    }
}
