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

// === Implementations for management decision templates and models ===

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

// === Implementations for management incidents templates and models ===

impl TemplateRealize for ManagementIncidentTemplateDef {
    type Model = ManagementIncidentModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        use ManagementIncidentTemplateDef::*;
        type M = ManagementIncidentModel;

        match self {
            MislabelPrice(def) => M::MislabelPrice(def.realize(ctx)),
        }
    }
}

impl ViewParams for ManagementIncidentModel {
    fn params(&self) -> ParamsMap {
        use ManagementIncidentModel::*;
        match self {
            MislabelPrice(model) => model.params(),
        }
    }
}

impl TemplateRealize for MislabelPriceTemplate {
    type Model = MislabelPriceModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let num_items = ctx.rng.random_range(self.num_range.clone());

        let mut rates = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            let overprice_rate = ctx.rng.random_range(self.overprice_rate_range.clone());
            rates.push(overprice_rate);
        }

        Self::Model {
            overpriced_rates: rates,
        }
    }
}

impl ViewParams for MislabelPriceModel {
    fn params(&self) -> ParamsMap {
        // we do not provide any specific params for this incident
        params! {}
    }
}
