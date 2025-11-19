mod apply_decision;
mod apply_incident;
mod convert;

use dishaster_views::{ManagementDecisionView, ManagementDecisionsView, ManagementIncidentView};

use self::convert::{RealizationContext, TemplateRealize, ViewParams};
pub use self::{
    apply_decision::register_management_decision_systems,
    apply_incident::register_management_incident_systems,
};
use crate::{events::*, systems::prelude::*};

/// Roll a set of management decisions to present to the player
pub fn roll_management_decisions(
    _event: On<RollManagementDecisions>,
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    day_status: Res<DayStatus>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
) {
    /// Number of management decisions to roll each time
    const DECISION_ROLL_COUNT: usize = 3;

    let mut rng = rng.derive_prng();
    let templates = registry.mgmt_decisions.iter().collect::<Vec<_>>();

    let mut models = vec![];
    let mut views = vec![];

    // Roll new decisions
    for _ in 0..DECISION_ROLL_COUNT {
        let Ok(template) = templates.choose_weighted(&mut rng, |t| t.weight) else {
            continue;
        };

        let ctx = RealizationContext { rng: &mut rng };
        let model = template.def.realize(ctx);
        views.push(ManagementDecisionView {
            model_id: template.id.clone(),
            params: model.params(),
            icon: template.icon.clone(),
        });
        models.push(model);
    }

    events.push(SimEvent::ShowManagementDecisions(
        ManagementDecisionsView {
            day: day_status.current_day.0,
            options: views,
        }
        .into(),
    ));
    // TODO: we may need to populate more data here for the view, such as options

    // Insert into resources
    commands.insert_resource(ManagementDecisions { available: models });
}

/// Apply a selected management decision
pub fn apply_management_decision(
    event: On<ApplyManagementDecision>,
    mut commands: Commands,
    mut decisions: ResMut<ManagementDecisions>,
) {
    let option_index = event.0;
    let model = std::mem::take(&mut decisions.available).remove(option_index);

    macro_rules! dispatch {
        ($($variant:ident),* $(,)?) => {
            match model {
                $( ManagementDecisionModel::$variant(model) => commands.trigger(DispatchManagement(model)), )*
            }
        };
    }

    dispatch!(
        AddTables,
        RemoveTables,
        DisarrangeTables,
        OpenWindow,
        CloseWindow,
        ChangeWindowService,
        PlayMusic,
        AdvertiseCampaign,
        AddMotivationalSlogan,
        SupplyCrab,
        ImproveDishQuality,
        ReduceServingTime,
    );

    // Remove decisions after applying
    commands.remove_resource::<ManagementDecisions>();

    // Advance to next day
    commands.trigger(AdvanceDay);
}

pub fn roll_management_incident(
    _event: On<RollManagementIncident>,
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
) {
    log::info!("Rolling management incident...");

    let mut rng = rng.derive_prng();
    let templates = registry.mgmt_incidents.iter().collect::<Vec<_>>();

    let Ok(template) = templates.choose_weighted(&mut rng, |t| t.weight) else {
        log::warn!("No management incident templates available to roll");
        return;
    };

    let ctx = RealizationContext { rng: &mut rng };
    let incident_model = template.def.realize(ctx);

    // Emit incident view event
    events.push(SimEvent::ShowManagementIncident(
        ManagementIncidentView {
            model_id: template.id.clone(),
            icon: template.icon.clone(),
            params: incident_model.params(),
        }
        .into(),
    ));

    macro_rules! dispatch {
        ($($variant:ident),* $(,)?) => {
            match incident_model {
                $( ManagementIncidentModel::$variant(model) => commands.trigger(DispatchManagement(model)), )*
            }
        };
    }

    dispatch!(
        MislabelPrice,
        AttractionChange,
        TemporaryCrowd,
        InspectorVisit,
    );
}
