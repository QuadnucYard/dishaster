//! Profile Validation

use dishaster_models::*;
use dishrupt_core::prelude::{HasId, ModelRegistry};

use crate::{ValidationError, ValidationResult};

/// Validates a player profile against the game model registry
///
/// Ensures all model references in the profile (level, dishes, services) exist.
pub fn validate_player_profile(
    profile: &dishaster_save_models::PlayerProfile,
    registry: &GameModelRegistry,
) -> ValidationResult {
    // Validate level reference
    if !registry.levels.contains_id(&profile.progress.level_id) {
        return Err(ValidationError::MissingReference {
            model_type: "level",
            id: profile.progress.level_id.clone(),
            context: "PlayerProgress.level_id".to_string(),
        });
    }

    // Validate canteen layout
    validate_canteen_layout(&profile.layout, registry)?;

    // Validate diner pool
    for (idx, diner) in profile.diner_pool.profiles.iter().enumerate() {
        validate_diner_profile(diner, registry, idx)?;
    }

    // Validate permanent effects
    validate_permanent_effects(&profile.permanent_effects, registry)?;

    Ok(())
}

/// Validates canteen layout state references
fn validate_canteen_layout(
    layout: &dishaster_save_models::CanteenLayoutState,
    registry: &GameModelRegistry,
) -> ValidationResult {
    // Validate window configurations
    for (idx, window_config) in layout.window_configurations.iter().enumerate() {
        let context = format!("window_configurations[{}]", idx);

        // Validate service reference
        if !registry.window_services.contains_id(&window_config.service) {
            return Err(ValidationError::MissingReference {
                model_type: "window_service",
                id: window_config.service.clone(),
                context: format!("{}.service", context),
            });
        }

        // Validate price override dish references
        for dish_id in window_config.price_override.keys() {
            if !registry.dishes.contains_id(dish_id) {
                return Err(ValidationError::MissingReference {
                    model_type: "dish",
                    id: dish_id.clone(),
                    context: format!("{}.price_override", context),
                });
            }
        }
    }

    // Validate placement references
    validate_placement_refs(
        &layout.placement.tables,
        "table",
        registry,
        &registry.tables,
    )?;
    validate_placement_refs(
        &layout.placement.tray_dispensers,
        "dispenser",
        registry,
        &registry.dispensers,
    )?;
    validate_placement_refs(
        &layout.placement.chopstick_dispensers,
        "dispenser",
        registry,
        &registry.dispensers,
    )?;
    validate_placement_refs(
        &layout.placement.collectors,
        "collector",
        registry,
        &registry.collectors,
    )?;

    Ok(())
}

/// Validates placement model references
fn validate_placement_refs<T: HasId>(
    placements: &[dishaster_save_models::Placement],
    model_type: &'static str,
    _registry: &GameModelRegistry,
    models: &ModelRegistry<T>,
) -> ValidationResult {
    for (idx, placement) in placements.iter().enumerate() {
        if !models.contains_id(&placement.model) {
            return Err(ValidationError::MissingReference {
                model_type,
                id: placement.model.clone(),
                context: format!("placement[{}]", idx),
            });
        }
    }
    Ok(())
}

/// Validates a diner profile's model references
fn validate_diner_profile(
    diner: &dishaster_save_models::DinerProfile,
    registry: &GameModelRegistry,
    index: usize,
) -> ValidationResult {
    // Validate dish experience references
    for dish_id in diner.long_term_memory.dish_experience.keys() {
        if !registry.dishes.contains_id(dish_id) {
            return Err(ValidationError::MissingReference {
                model_type: "dish",
                id: dish_id.clone(),
                context: format!(
                    "diner_pool.profiles[{}].long_term_memory.dish_experience",
                    index
                ),
            });
        }
    }

    Ok(())
}

/// Validates permanent effects model references
fn validate_permanent_effects(
    effects: &dishaster_save_models::PermanentEffects,
    registry: &GameModelRegistry,
) -> ValidationResult {
    // Validate luxury dish references
    for dish_id in &effects.luxury_dishes {
        if !registry.dishes.contains_id(dish_id) {
            return Err(ValidationError::MissingReference {
                model_type: "dish",
                id: dish_id.clone(),
                context: "permanent_effects.luxury_dishes".to_string(),
            });
        }
    }

    // Validate campaign window references
    for (idx, campaign) in effects.campaigns.iter().enumerate() {
        if let dishaster_save_models::CampaignTarget::Window(window_id) = &campaign.target
            && !registry.window_services.contains_id(window_id)
        {
            return Err(ValidationError::MissingReference {
                model_type: "window_service",
                id: window_id.clone(),
                context: format!("permanent_effects.campaigns[{}].target", idx),
            });
        }
    }

    Ok(())
}
