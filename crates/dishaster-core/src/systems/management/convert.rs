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
        macro_rules! dispatch {
            ($($variant:ident),* $(,)?) => {
            match self {
                    $( ManagementDecisionTemplateDef::$variant(def) => ManagementDecisionModel::$variant(def.realize(ctx)), )*
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
            AddLuxuryDish,
        )
    }
}

impl ViewParams for ManagementDecisionModel {
    fn params(&self) -> ParamsMap {
        macro_rules! dispatch {
            ($($variant:ident),* $(,)?) => {
                match self {
                    $( ManagementDecisionModel::$variant(model) => model.params(), )*
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
            AddLuxuryDish,
        )
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

impl TemplateRealize for OpenWindowTemplate {
    type Model = OpenWindowModel;

    fn realize(&self, _ctx: RealizationContext) -> Self::Model {
        Self::Model {}
    }
}

impl ViewParams for OpenWindowModel {
    fn params(&self) -> ParamsMap {
        params! {}
    }
}

impl TemplateRealize for CloseWindowTemplate {
    type Model = CloseWindowModel;

    fn realize(&self, _ctx: RealizationContext) -> Self::Model {
        Self::Model {}
    }
}

impl ViewParams for CloseWindowModel {
    fn params(&self) -> ParamsMap {
        params! {}
    }
}

impl TemplateRealize for ChangeWindowServiceTemplate {
    type Model = ChangeWindowServiceModel;

    fn realize(&self, _ctx: RealizationContext) -> Self::Model {
        Self::Model {}
    }
}

impl ViewParams for ChangeWindowServiceModel {
    fn params(&self) -> ParamsMap {
        params! {}
    }
}

impl TemplateRealize for PlayMusicTemplate {
    type Model = PlayMusicModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let eating_time_multiplier = ctx
            .rng
            .random_range(self.eating_time_multiplier_range.clone());
        let satisfaction_change = ctx.rng.random_range(self.satisfaction_change_range.clone());
        Self::Model {
            eating_time_multiplier,
            satisfaction_change,
        }
    }
}

impl ViewParams for PlayMusicModel {
    fn params(&self) -> ParamsMap {
        // Calculate percentage change for display
        let speed_change = ((1.0 / self.eating_time_multiplier - 1.0) * 100.0).round() as i32;
        let satisfaction_change = (self.satisfaction_change * 100.0).round() as i32;

        params! {
            speed_change => speed_change.abs(),
            satisfaction_change => if satisfaction_change >= 0 {
                eco_format!("+{}", satisfaction_change)
            } else {
                eco_format!("{}", satisfaction_change)
            },
        }
    }
}

impl TemplateRealize for AdvertiseCampaignTemplate {
    type Model = AdvertiseCampaignModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let attraction_boost = ctx.rng.random_range(self.attraction_boost_range.clone());
        // Target window will be randomly selected when applying the decision
        Self::Model {
            target: self.target.clone(),
            attraction_boost,
            days_remaining: self.duration_days,
            decay_rate: self.decay_rate,
            target_window: None,
        }
    }
}

impl ViewParams for AdvertiseCampaignModel {
    fn params(&self) -> ParamsMap {
        // Calculate percentage values for display
        let boost_percent = ((self.attraction_boost - 1.0) * 100.0).round() as i32;
        let decay_percent = (self.decay_rate * 100.0).round() as i32;

        params! {
            target => match &self.target {
                DecisionCampaignTarget::Canteen => "canteen",
                DecisionCampaignTarget::Window => "window",
            },
            boost => boost_percent,
            days => self.days_remaining,
            decay => decay_percent,
        }
    }
}

impl TemplateRealize for AddMotivationalSloganTemplate {
    type Model = AddMotivationalSloganModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let trust_threshold = ctx.rng.random_range(self.trust_threshold_range.clone());
        let satisfaction_boost = ctx.rng.random_range(self.satisfaction_boost_range.clone());
        let satisfaction_penalty = ctx
            .rng
            .random_range(self.satisfaction_penalty_range.clone());
        Self::Model {
            trust_threshold,
            satisfaction_boost,
            satisfaction_penalty,
        }
    }
}

impl ViewParams for AddMotivationalSloganModel {
    fn params(&self) -> ParamsMap {
        // Convert to percentage and format
        let threshold_percent = (self.trust_threshold * 100.0).round() as i32;
        let boost_percent = (self.satisfaction_boost * 100.0).round() as i32;
        let penalty_percent = (self.satisfaction_penalty * 100.0).round() as i32;

        params! {
            threshold => threshold_percent,
            boost => eco_format!("+{}", boost_percent),
            penalty => eco_format!("{}", penalty_percent),
        }
    }
}

impl TemplateRealize for AddLuxuryDishTemplate {
    type Model = AddLuxuryDishModel;

    fn realize(&self, _ctx: RealizationContext) -> Self::Model {
        Self::Model {
            dish_id: self.dish_id.clone(),
            applied: false,
        }
    }
}

impl ViewParams for AddLuxuryDishModel {
    fn params(&self) -> ParamsMap {
        params! {
            dish_id => self.dish_id.clone().to_string(),
            applied => if self.applied { 1 } else { 0 },
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
