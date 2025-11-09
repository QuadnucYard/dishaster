use dishaster_views::{ManagementDecisionView, ManagementDecisionsView};

use crate::{
    events::*,
    systems::{prelude::*, spawn_table},
};

struct RealizationContext<'a> {
    pub rng: &'a mut Prng,
}

trait TemplateRealize {
    type Model;

    fn realize(&self, ctx: RealizationContext) -> Self::Model;
}

impl TemplateRealize for ManagementDecisionTemplateDef {
    type Model = ManagementDecisionModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        use ManagementDecisionTemplateDef::*;
        type M = ManagementDecisionModel;

        match self {
            AddTables(def) => M::AddTables(def.realize(ctx)),
            RemoveTables(def) => M::RemoveTables(def.realize(ctx)),
            DisarrangeTables(def) => M::DisarrangeTables(def.realize(ctx)),
        }
    }
}

impl TemplateRealize for AddTablesTemplate {
    type Model = AddTablesModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let num_tables = ctx.rng.random_range(self.num_range.clone());
        Self::Model { num_tables }
    }
}

impl TemplateRealize for RemoveTablesTemplate {
    type Model = RemoveTablesModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let num_tables = ctx.rng.random_range(self.num_range.clone());
        Self::Model { num_tables }
    }
}

impl TemplateRealize for DisarrangeTablesTemplate {
    type Model = DisarrangeTablesModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let num_tables = ctx.rng.random_range(self.num_range.clone());
        Self::Model { num_tables }
    }
}

/// Roll a set of management decisions to present to the player
pub fn roll_management_decisions(
    _event: On<RollManagementDecisions>,
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    level: Res<ResWrapper<LevelSetupState>>,
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
        let Some(template) = templates.choose_weighted(&mut rng, |t| t.weight).ok() else {
            continue;
        };

        let ctx = RealizationContext { rng: &mut rng };
        decisions.push((template.id.clone(), template.def.realize(ctx)));
    }

    events.push(SimEvent::ShowManagementDecisions(
        ManagementDecisionsView {
            day: level.day,
            options: decisions
                .iter()
                .map(|(id, _model)| ManagementDecisionView {
                    model_id: id.clone(),
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
    canteen: Res<ResWrapper<Canteen>>,
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
}

fn random_table_position(canteen: &CanteenModel, rng: &mut Prng) -> Vec2 {
    // TODO: better placement logic
    vec2(
        rng.random_range(1.0..(canteen.width - 1.0)),
        rng.random_range(1.0..(canteen.windows_y - 1.0)),
    )
}
