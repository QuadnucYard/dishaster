//! Conversion logic for management decision and incident templates to models

use dishaster_views::{ParamsMap, params};

use crate::systems::prelude::*;

/// Context for realizing templates into models
pub struct RealizationContext<'a> {
    /// Random number generator
    pub rng: &'a mut Prng,
}

/// Trait for realizing a template into a model
pub trait TemplateRealize {
    /// The resulting model type
    type Model;

    /// Realize the template into a model given the context
    fn realize(&self, ctx: RealizationContext) -> Self::Model;
}

/// Trait for obtaining view parameters from a model, used for localization
pub trait ViewParams {
    /// The parameters map for view
    fn params(&self) -> ParamsMap;
}

// === Implementations for management decision templates and models ===

impl TemplateRealize for ManagementDecisionTemplateDef {
    type Model = ManagementDecisionModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        macro_rules! dispatch {
            ($($variant:ident),* $(,)?) => {
            match self {
                    $( Self::$variant(def) => Self::Model::$variant(def.realize(ctx)), )*
                }
            };
        }

        dispatch!(
            AddTables,
            RemoveTables,
            DisarrangeTables,
            AddDispenser,
            OpenWindow,
            CloseWindow,
            ChangeWindowService,
            PlayMusic,
            AdvertiseCampaign,
            AddMotivationalSlogan,
            SupplyCrab,
            ImproveDishQuality,
            ReduceServingTime,
        )
    }
}

impl ViewParams for ManagementDecisionModel {
    fn params(&self) -> ParamsMap {
        macro_rules! dispatch {
            ($($variant:ident),* $(,)?) => {
                match self {
                    $( Self::$variant(model) => model.params(), )*
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
            AddDispenser,
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

impl TemplateRealize for AddDispenserTemplate {
    type Model = AddDispenserModel;

    fn realize(&self, _ctx: RealizationContext) -> Self::Model {
        Self::Model {
            dispenser_type: self.dispenser_type,
            dispenser_model: self.dispenser_model.clone(),
        }
    }
}

impl ViewParams for AddDispenserModel {
    fn params(&self) -> ParamsMap {
        params! {}
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
        let satisfaction_change = (self.satisfaction_change * 100.0).round() as i32;

        params! {
            speed_change => (1.0 / self.eating_time_multiplier - 1.0).abs(),
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
        params! {
            target => match &self.target {
                DecisionCampaignTarget::Canteen => "canteen",
                DecisionCampaignTarget::Window => "window",
            },
            boost => self.attraction_boost - 1.0,
            days => self.days_remaining,
            decay => self.decay_rate,
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
        params! {
            threshold => self.trust_threshold,
            boost => self.satisfaction_boost,
            penalty => self.satisfaction_penalty,
        }
    }
}

impl TemplateRealize for SupplyCrabTemplate {
    type Model = SupplyCrabModel;

    fn realize(&self, _ctx: RealizationContext) -> Self::Model {
        Self::Model {
            trial_probability: self.trial_probability,
        }
    }
}

impl ViewParams for SupplyCrabModel {
    fn params(&self) -> ParamsMap {
        params! {}
    }
}

impl TemplateRealize for ImproveDishQualityTemplate {
    type Model = ImproveDishQualityModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let quality_multiplier = ctx.rng.random_range(self.quality_multiplier_range.clone());
        Self::Model { quality_multiplier }
    }
}

impl ViewParams for ImproveDishQualityModel {
    fn params(&self) -> ParamsMap {
        params! {
            improvement => self.quality_multiplier - 1.0,
        }
    }
}

impl TemplateRealize for ReduceServingTimeTemplate {
    type Model = ReduceServingTimeModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let serving_time_multiplier = ctx
            .rng
            .random_range(self.serving_time_multiplier_range.clone());
        Self::Model {
            serving_time_multiplier,
        }
    }
}

impl ViewParams for ReduceServingTimeModel {
    fn params(&self) -> ParamsMap {
        params! {
            reduction => 1.0 - self.serving_time_multiplier,
        }
    }
}

// === Implementations for management incidents templates and models ===

impl TemplateRealize for ManagementIncidentTemplateDef {
    type Model = ManagementIncidentModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        macro_rules! dispatch {
            ($($variant:ident),* $(,)?) => {
            match self {
                    $( Self::$variant(def) => Self::Model::$variant(def.realize(ctx)), )*
                }
            };
        }

        dispatch!(
            MislabelPrice,
            AttractionChange,
            TemporaryCrowd,
            InspectorVisit,
        )
    }
}

impl ViewParams for ManagementIncidentModel {
    fn params(&self) -> ParamsMap {
        macro_rules! dispatch {
            ($($variant:ident),* $(,)?) => {
                match self {
                    $( Self::$variant(model) => model.params(), )*
                }
            };
        }

        dispatch!(
            MislabelPrice,
            AttractionChange,
            TemporaryCrowd,
            InspectorVisit,
        )
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

impl TemplateRealize for AttractionChangeTemplate {
    type Model = AttractionChangeModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        Self::Model {
            attraction_multiplier: ctx
                .rng
                .random_range(self.attraction_multiplier_range.clone()),
        }
    }
}

impl ViewParams for AttractionChangeModel {
    fn params(&self) -> ParamsMap {
        // Notification doesn't need to display the parameter
        params! {}
    }
}

impl TemplateRealize for TemporaryCrowdTemplate {
    type Model = TemporaryCrowdModel;

    fn realize(&self, ctx: RealizationContext) -> Self::Model {
        let num_diners = ctx.rng.random_range(self.num_diners_range.clone());
        let peak_time = ctx.rng.random_range(self.peak_time_range.clone());

        Self::Model {
            peak_time,
            num_diners,
            time_stddev: self.time_stddev,
        }
    }
}

impl ViewParams for TemporaryCrowdModel {
    fn params(&self) -> ParamsMap {
        // Notification doesn't need to display the parameter
        params! {}
    }
}

impl TemplateRealize for InspectorVisitTemplate {
    type Model = InspectorVisitModel;

    fn realize(&self, _ctx: RealizationContext) -> Self::Model {
        // Since the execution is deferred until later, we just copy the parameters here
        Self::Model {
            fsri_threshold: self.fsri_threshold,
            probability_multiplier: self.probability_multiplier,
            reputation_boost: self.reputation_boost,
            trust_boost: self.trust_boost,
        }
    }
}

impl ViewParams for InspectorVisitModel {
    fn params(&self) -> ParamsMap {
        // Notification doesn't need to display the parameter
        params! {}
    }
}
