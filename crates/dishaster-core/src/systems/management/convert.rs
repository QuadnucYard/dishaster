use dishaster_views::{ParamsMap, params};

use crate::systems::prelude::*;

/// Context for realizing templates into models
pub struct RealizationContext<'a> {
    pub rng: &'a mut Prng,
}

/// Trait for realizing a template into a model
pub trait TemplateRealize {
    type Model;

    fn realize(&self, ctx: RealizationContext) -> Self::Model;
}

/// Trait for obtaining view parameters from a model, used for localization
pub trait ViewParams {
    fn params(&self) -> ParamsMap;
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

impl ViewParams for ManagementDecisionModel {
    fn params(&self) -> ParamsMap {
        use ManagementDecisionModel::*;
        match self {
            AddTables(model) => model.params(),
            RemoveTables(model) => model.params(),
            DisarrangeTables(model) => model.params(),
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

impl ViewParams for AddTablesModel {
    fn params(&self) -> ParamsMap {
        params! {
            num_tables => self.num_tables,
        }
    }
}

impl TemplateRealize for RemoveTablesTemplate {
    type Model = RemoveTablesModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let num_tables = ctx.rng.random_range(self.num_range.clone());
        Self::Model { num_tables }
    }
}

impl ViewParams for RemoveTablesModel {
    fn params(&self) -> ParamsMap {
        params! {
            num_tables => self.num_tables,
        }
    }
}

impl TemplateRealize for DisarrangeTablesTemplate {
    type Model = DisarrangeTablesModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let num_tables = ctx.rng.random_range(self.num_range.clone());
        Self::Model { num_tables }
    }
}

impl ViewParams for DisarrangeTablesModel {
    fn params(&self) -> ParamsMap {
        params! {
            num_tables => self.num_tables,
        }
    }
}
