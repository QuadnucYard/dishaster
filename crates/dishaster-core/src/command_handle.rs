use bevy_ecs::system::RunSystemOnce;
use dishaster_channel::{commands::SimCommand, events::*};
use dishaster_models::{
    TrialIntro, TrialParticipantAppearance, TrialResponseOption, TrialSpeech, TrialSpeechItem,
    TrialStatement,
};
use dishaster_navigation::*;

use crate::{components::*, prelude::*, resources::*, sim::Simulation};

impl Simulation {
    /// Apply a high-level control command from the client runtime.
    ///
    /// Commands alter stateful resources directly so that the next simulation tick
    /// reflects the requested transition without delay.
    pub(crate) fn handle_command_impl(&mut self, command: SimCommand) {
        match command {
            SimCommand::SetDebugFlags(debug_flags) => {
                self.debug_flags = debug_flags;
            }

            SimCommand::StartRun => {
                let mut day_status = self.world.resource_mut::<DayStatus>();
                day_status.started = true;
            }
            SimCommand::EndRun => {
                // stop spawning
                let mut spawner = self.world.resource_mut::<DinerSpawner>();
                spawner.spawning_finished = true;

                // clear diners
                let _ = self.world.run_system_once(
                    |mut commands: Commands, query: Query<Entity, With<Diner>>| {
                        for entity in query.iter() {
                            commands.entity(entity).despawn();
                        }
                    },
                );
            }

            SimCommand::UpdateDishPricing {
                dish_entity,
                pricing,
            } => {
                if let Some(mut dish) = self.world.get_mut::<Dish>(dish_entity.into()) {
                    dish.pricing = pricing;
                }
            }

            SimCommand::QueryDistance(pos) => {
                let resp = {
                    let nav_grid = self.world.resource_mut::<ResWrapper<NavigationGrid>>();
                    nav_grid
                        .try_world_to_grid(pos)
                        .map(|cell| nav_grid.get_distance(cell))
                };
                let mut events = self.world.resource_mut::<EventLog>();
                events.emit(PresentationEvent::QueryDistanceResponse(resp));
            }
            SimCommand::QueryDistances => {
                let resp = {
                    let nav_grid = self.world.resource_mut::<ResWrapper<NavigationGrid>>();
                    let distances = nav_grid.distance_field();
                    QueryDistancesResponse {
                        width: distances.rows(),
                        height: distances.cols(),
                        cell_size: nav_grid.cell_size(),
                        data: distances.flatten().clone(),
                    }
                };
                let mut events = self.world.resource_mut::<EventLog>();
                events.emit(PresentationEvent::QueryDistancesResponse(resp));
            }

            SimCommand::TrialStart(_entity_id) => {
                // Currently, emit random appearances for both sides.
                let intro = {
                    let mut rng = self.world.resource_mut::<GameRng>();
                    TrialIntro {
                        left: random_appearance(&mut rng),
                        right: random_appearance(&mut rng),
                    }
                };

                let mut events = self.world.resource_mut::<EventLog>();
                events.emit(PresentationEvent::TrialIntro(intro));
            }
            SimCommand::TrialLaunch => {
                let statement = self.create_diner_statement();

                let mut events = self.world.resource_mut::<EventLog>();
                events.emit(PresentationEvent::TrialLeftSpeak(statement));
            }
            SimCommand::TrialRespond(_resp_corpus_index) => {
                // For now, just respond with a random speech for the right side.
                let speech = self.launch_trial_speech(true);

                let mut events = self.world.resource_mut::<EventLog>();
                events.emit(PresentationEvent::TrialRightSpeak(speech));
            }
            SimCommand::TrialTimeout => {
                // TODO: Simply end the trial on timeout for now.
                let mut events = self.world.resource_mut::<EventLog>();
                events.emit(PresentationEvent::TrialEnd);
            }
            SimCommand::TrialProceed => {
                let should_continue = {
                    let mut rng = self.world.resource_mut::<GameRng>();
                    rng.random_bool(0.5)
                };

                if should_continue {
                    // here the logic is same as `TrialLaunch` for now
                    let statement = self.create_diner_statement();

                    let mut events = self.world.resource_mut::<EventLog>();
                    events.emit(PresentationEvent::TrialLeftSpeak(statement));
                } else {
                    // End of trial
                    let mut events = self.world.resource_mut::<EventLog>();
                    events.emit(PresentationEvent::TrialEnd);
                }
            }
        }
    }

    fn create_diner_statement(&mut self) -> TrialStatement {
        let speech = self.launch_trial_speech(false);
        let options = self.prepare_trial_response_options(&speech);
        TrialStatement { speech, options }
    }

    fn prepare_trial_response_options(
        &mut self,
        speech: &TrialSpeech,
    ) -> Vec<Vec<TrialResponseOption>> {
        (self.world).resource_scope(|world, registry: Mut<GameModelRegistryRes>| {
            let mut options = Vec::new();

            let mut rng = world.resource_mut::<GameRng>();
            for item in &speech.items {
                let TrialSpeechItem::Keyword(_keyword) = item else {
                    continue;
                };
                // For simplicity, pick a random response for each keyword.
                let options_count = rng.random_range(1..=3);
                options.push(
                    (0..registry.trial.responses.len())
                        .choose_multiple(&mut rng, options_count)
                        .iter()
                        .map(|&idx| {
                            let response = registry.trial.responses.get(idx).cloned().unwrap();
                            TrialResponseOption {
                                corpus_index: idx,
                                kind: response.kind,
                                summary: response.summary.clone(),
                            }
                        })
                        .collect(),
                );
            }

            options
        })
    }

    fn launch_trial_speech(&mut self, is_player: bool) -> TrialSpeech {
        self.world
            .run_system_once(
                move |registry: Res<GameModelRegistryRes>, mut rng: ResMut<GameRng>| {
                    if is_player {
                        registry
                            .trial
                            .responses
                            .choose(&mut rng)
                            .cloned()
                            .unwrap()
                            .content
                    } else {
                        registry
                            .trial
                            .diner_speeches
                            .choose(&mut rng)
                            .cloned()
                            .unwrap()
                    }
                },
            )
            .unwrap()
    }
}

fn random_appearance(rng: &mut GameRng) -> TrialParticipantAppearance {
    TrialParticipantAppearance {
        emotion: [
            '😅', '😡', '😠', '😤', '😞', '😢', '😭', '😰', '😨', '😱', '😠',
        ]
        .choose(rng)
        .copied()
        .unwrap(),
        gesture: [
            '👍', '👎', '👊', '🤚', '✋', '👋', '🤞', '🤏', '👈', '👉', '🤝', '👍', '👏', '🤌',
        ]
        .choose(rng)
        .copied(),
    }
}
