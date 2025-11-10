mod convert;

use dishaster_views::{ManagementDecisionView, ManagementDecisionsView, ManagementIncidentView};

use self::convert::{RealizationContext, TemplateRealize, ViewParams};
use crate::{
    events::*,
    systems::{prelude::*, spawn_table},
};

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

    let mut decisions = vec![];

    // Roll new decisions
    for _ in 0..DECISION_ROLL_COUNT {
        let Ok(template) = templates.choose_weighted(&mut rng, |t| t.weight) else {
            continue;
        };

        let ctx = RealizationContext { rng: &mut rng };
        decisions.push((template.id.clone(), template.def.realize(ctx)));
    }

    events.push(SimEvent::ShowManagementDecisions(
        ManagementDecisionsView {
            day: day_status.current_day.0,
            options: decisions
                .iter()
                .map(|(id, model)| ManagementDecisionView {
                    model_id: id.clone(),
                    params: model.params(),
                })
                .collect(),
        }
        .into(),
    ));

    // Insert into resources
    commands.insert_resource(ManagementDecisions {
        available: decisions.into_iter().map(|(_, m)| m).collect(),
    });
}

/// Apply a selected management decision
pub fn apply_management_decision(
    event: On<ApplyManagementDecision>,
    mut commands: Commands,
    mut table_query: Query<(Entity, &mut DiningTable, &mut Transform)>,
    registry: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    display_root: Res<DisplayRoot>,
    decisions: Res<ManagementDecisions>,
    mut rng: ResMut<WorldRng>,
) {
    use ManagementDecisionModel::*;

    let option_index = event.0;
    let model = decisions
        .available
        .get(option_index)
        .expect("Invalid management decision index");

    let mut rng = rng.derive_prng();
    match &model {
        AddTables(model) => {
            let mut cnt = 0;
            while cnt < model.num_tables {
                // Spawn at random position
                let table_model_id = registry
                    .tables
                    .keys()
                    .choose(&mut rng)
                    .expect("No table models in registry")
                    .clone();
                let center_pos = random_table_position(&canteen.model, &mut rng);

                // TODO: check for collisions with existing tables
                let placement = Placement {
                    model: table_model_id,
                    center_pos,
                };
                spawn_table(&placement, &mut commands, &registry, &display_root);

                cnt += 1;
            }
        }
        RemoveTables(model) => {
            for (entity, _, _) in table_query
                .iter()
                .choose_multiple(&mut rng, model.num_tables)
            {
                commands.entity(entity).despawn();
            }
        }
        DisarrangeTables(model) => {
            for (_, mut table, mut transform) in table_query
                .iter_mut()
                .choose_multiple(&mut rng, model.num_tables)
            {
                let new_pos = random_table_position(&canteen.model, &mut rng);
                table.center_pos = new_pos;
                transform.position = new_pos.extend(0.);
            }
        }
    }

    // Remove decisions after applying
    commands.remove_resource::<ManagementDecisions>();

    // Advance to next day
    commands.trigger(AdvanceDay);
}

fn random_table_position(canteen: &CanteenModel, rng: &mut Prng) -> Vec2 {
    // TODO: better placement logic
    vec2(
        rng.random_range(1.0..(canteen.width - 1.0)),
        rng.random_range(1.0..(canteen.windows_y - 1.0)),
    )
}

pub fn roll_management_incident(
    _event: On<RollManagementIncident>,
    registry: Res<GameModelRegistryRes>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
    mut dish_query: Query<(Entity, &mut Dish)>,
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
            params: incident_model.params(),
        }
        .into(),
    ));

    // Apply incident effects here as needed
    match incident_model {
        ManagementIncidentModel::MislabelPrice(model) => {
            for rate in model.overpriced_rates {
                // Choose a dish to mislabel
                let Some((dish_entity, mut dish)) = dish_query.iter_mut().choose(&mut rng) else {
                    continue;
                };

                let (original_price, new_price) = match &mut dish.assignment.pricing {
                    PricingMethod::PerPortion(v) | PricingMethod::ByWeight(v) => {
                        let original_price = *v;
                        let new_price = original_price * (1.0 + rate);
                        *v = new_price;
                        (original_price, new_price)
                    }
                };

                log::info!(
                    "Mislabeling price for dish {:?}: {:.2} -> {:.2}",
                    dish.assignment.dish_id,
                    original_price,
                    new_price
                );

                // todo: we need to know the dish entity
                events.push(SimEvent::DishPriceChanged {
                    entity: dish_entity.to_entity_id(),
                    new_pricing: dish.assignment.pricing.to_view(),
                });
            }
        }
    }
}
