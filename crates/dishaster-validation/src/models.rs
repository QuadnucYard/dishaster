//! Data validation for game model integrity
//!
//! Ensures all model references are valid and configurations are consistent.

use std::collections::HashSet;

use dishaster_models::*;
use dishrupt_core::prelude::{HasId, ModelRegistry};

use crate::{ValidationError, ValidationResult};

/// Validates the entire game model registry
///
/// Performs comprehensive checks to ensure:
/// - All model references are valid
/// - No duplicate IDs exist
/// - Required fields are populated
/// - Value ranges are reasonable
pub fn validate_registry(registry: &GameModelRegistry) -> ValidationResult {
    let mut errors = Vec::new();

    // Validate each model type
    if let Err(e) = validate_levels(registry) {
        errors.push(e);
    }
    if let Err(e) = validate_window_services(registry) {
        errors.push(e);
    }
    if let Err(e) = validate_dishes(registry) {
        errors.push(e);
    }
    if let Err(e) = validate_canteens(registry) {
        errors.push(e);
    }

    // Report first error if any exist
    if let Some(first_error) = errors.first() {
        return Err(first_error.clone());
    }

    Ok(())
}

/// Validate level configurations
fn validate_levels(registry: &GameModelRegistry) -> ValidationResult {
    for level in registry.levels.iter() {
        let context = format!("level '{}'", level.id);

        // Check canteen reference
        if !registry.canteens.contains_id(&level.canteen) {
            return Err(ValidationError::MissingReference {
                model_type: "canteen",
                id: level.canteen.clone(),
                context: context.clone(),
            });
        }

        // Validate window configurations
        for (idx, window_config) in level.window_configurations.iter().enumerate() {
            let window_context = format!("{}, window_config[{}]", context, idx);

            // Check service reference
            if !registry.window_services.contains_id(&window_config.service) {
                return Err(ValidationError::MissingReference {
                    model_type: "window_service",
                    id: window_config.service.clone(),
                    context: window_context.clone(),
                });
            }

            // Validate price overrides reference valid dishes
            for dish_id in window_config.price_override.keys() {
                if !registry.dishes.contains_id(dish_id) {
                    return Err(ValidationError::MissingReference {
                        model_type: "dish",
                        id: dish_id.clone(),
                        context: format!("{}, price_override", window_context),
                    });
                }
            }
        }

        // Validate placements
        validate_placements(&level.table_placements, &registry.tables, "table", &context)?;
        validate_placements(
            &level.tray_dispenser_placements,
            &registry.dispensers,
            "dispenser",
            &context,
        )?;
        validate_placements(
            &level.chopstick_dispenser_placements,
            &registry.dispensers,
            "dispenser",
            &context,
        )?;
        validate_placements(
            &level.collector_placements,
            &registry.collectors,
            "collector",
            &context,
        )?;

        // Validate basic value ranges
        if level.run_length <= 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "run_length",
                context: context.clone(),
                reason: "must be positive".to_string(),
            });
        }

        // Validate diner randomizer
        validate_diner_randomizer(&level.diner_randomizer).map_err(|e| {
            // Wrap the error with context
            match e {
                ValidationError::InvalidValue {
                    field,
                    context: _,
                    reason,
                } => ValidationError::InvalidValue {
                    field,
                    context: format!("{}, diner_randomizer", context),
                    reason,
                },
                other => other,
            }
        })?;
    }

    Ok(())
}

/// Validate window service models
fn validate_window_services(registry: &GameModelRegistry) -> ValidationResult {
    for service in registry.window_services.iter() {
        let context = format!("window_service '{}'", service.id);

        // Check if dish_options is empty
        if service.dish_options.is_empty() {
            return Err(ValidationError::EmptyCollection {
                collection: "dish_options",
                context: context.clone(),
            });
        }

        // Validate all dish references
        for (idx, priced_dish) in service.dish_options.iter().enumerate() {
            if !registry.dishes.contains_id(&priced_dish.dish_id) {
                return Err(ValidationError::MissingReference {
                    model_type: "dish",
                    id: priced_dish.dish_id.clone(),
                    context: format!("{}, dish_options[{}]", context, idx),
                });
            }

            // Validate pricing values
            match priced_dish.pricing {
                PricingMethod::PerPortion(price) if price < 0.0 => {
                    return Err(ValidationError::InvalidValue {
                        field: "price",
                        context: format!("{}, dish_options[{}]", context, idx),
                        reason: "price cannot be negative".to_string(),
                    });
                }
                PricingMethod::ByWeight(price_per_kg) if price_per_kg < 0.0 => {
                    return Err(ValidationError::InvalidValue {
                        field: "price_per_kg",
                        context: format!("{}, dish_options[{}]", context, idx),
                        reason: "price cannot be negative".to_string(),
                    });
                }
                _ => {}
            }
        }

        // Validate layout
        if service.layout.queue_x.is_empty() {
            return Err(ValidationError::EmptyCollection {
                collection: "queue_x",
                context: context.clone(),
            });
        }

        // Validate queue positions are reasonable
        for (idx, &x_pos) in service.layout.queue_x.iter().enumerate() {
            if x_pos < 0.0 {
                return Err(ValidationError::InvalidValue {
                    field: "queue_x",
                    context: format!("{}, queue_x[{}]", context, idx),
                    reason: "position cannot be negative".to_string(),
                });
            }
        }
    }

    Ok(())
}

/// Validate dish models
fn validate_dishes(registry: &GameModelRegistry) -> ValidationResult {
    let mut seen_ids = HashSet::new();

    for dish in registry.dishes.iter() {
        let context = format!("dish '{}'", dish.id);

        // Check for duplicate IDs
        if !seen_ids.insert(dish.id.clone()) {
            return Err(ValidationError::DuplicateId {
                model_type: "dish",
                id: dish.id.clone(),
            });
        }

        // Validate quality range
        let quality_range = &dish.characteristics.quality_range;
        if quality_range.min > quality_range.max {
            return Err(ValidationError::InvalidValue {
                field: "quality_range",
                context: context.clone(),
                reason: format!("min ({}) > max ({})", quality_range.min, quality_range.max),
            });
        }

        if quality_range.min < 0.0 || quality_range.max > 1.0 {
            return Err(ValidationError::InvalidValue {
                field: "quality_range",
                context: context.clone(),
                reason: "quality must be in range [0.0, 1.0]".to_string(),
            });
        }

        // Validate risk level
        if dish.characteristics.risk_level < 0.0 || dish.characteristics.risk_level > 1.0 {
            return Err(ValidationError::InvalidValue {
                field: "risk_level",
                context: context.clone(),
                reason: "risk_level must be in range [0.0, 1.0]".to_string(),
            });
        }

        // Validate serving time
        if dish.characteristics.serving_time <= 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "serving_time",
                context: context.clone(),
                reason: "serving_time must be positive".to_string(),
            });
        }

        // Validate base price
        if dish.characteristics.base_price < 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "base_price",
                context: context.clone(),
                reason: "base_price cannot be negative".to_string(),
            });
        }

        // Validate weight distribution
        if dish.characteristics.weight_distrib.mean <= 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "weight_distrib.mean",
                context: context.clone(),
                reason: "mean weight must be positive".to_string(),
            });
        }

        if dish.characteristics.weight_distrib.stddev < 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "weight_distrib.stddev",
                context: context.clone(),
                reason: "stddev cannot be negative".to_string(),
            });
        }

        // Validate satiation per kg
        if dish.characteristics.satiation_per_kg <= 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "satiation_per_kg",
                context: context.clone(),
                reason: "satiation_per_kg must be positive".to_string(),
            });
        }

        // Validate eating time per kg
        if dish.characteristics.eating_time_per_kg <= 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "eating_time_per_kg",
                context: context.clone(),
                reason: "eating_time_per_kg must be positive".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate canteen models
fn validate_canteens(registry: &GameModelRegistry) -> ValidationResult {
    for canteen in registry.canteens.iter() {
        let context = format!("canteen '{}'", canteen.id);

        // Validate dimensions
        if canteen.width <= 0.0 || canteen.height <= 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "dimensions",
                context: context.clone(),
                reason: "width and height must be positive".to_string(),
            });
        }

        // Validate window slots exist
        if canteen.windows.is_empty() {
            return Err(ValidationError::EmptyCollection {
                collection: "windows",
                context: context.clone(),
            });
        }

        // Validate entrance ranges
        if canteen.entrances.is_empty() {
            return Err(ValidationError::EmptyCollection {
                collection: "entrances",
                context: context.clone(),
            });
        }

        for (idx, entrance) in canteen.entrances.iter().enumerate() {
            if entrance.x_min >= entrance.x_max {
                return Err(ValidationError::InvalidValue {
                    field: "entrance",
                    context: format!("{}, entrances[{}]", context, idx),
                    reason: format!("x_min ({}) >= x_max ({})", entrance.x_min, entrance.x_max),
                });
            }
        }
    }

    Ok(())
}

/// Helper to validate a collection of placements
fn validate_placements<T: HasId>(
    placements: &[Placement],
    registry: &ModelRegistry<T>,
    model_type: &'static str,
    context: &str,
) -> ValidationResult {
    for (idx, placement) in placements.iter().enumerate() {
        if !registry.contains_id(&placement.model) {
            return Err(ValidationError::MissingReference {
                model_type,
                id: placement.model.clone(),
                context: format!("{}, placement[{}]", context, idx),
            });
        }

        // Validate position is reasonable (not negative, not excessively large)
        if !((0.0..=1000.0).contains(&placement.center_pos.x)
            && (0.0..=1000.0).contains(&placement.center_pos.y))
        {
            return Err(ValidationError::InvalidValue {
                field: "center_pos",
                context: format!("{}, placement[{}]", context, idx),
                reason: format!(
                    "position ({:.1}, {:.1}) outside reasonable bounds [0, 1000]",
                    placement.center_pos.x, placement.center_pos.y
                ),
            });
        }
    }
    Ok(())
}

/// Validate diner randomizer configuration
pub fn validate_diner_randomizer(model: &DinerRandomizerModel) -> ValidationResult {
    let context = "DinerRandomizerModel";

    // Validate capacity ranges
    validate_range(
        &model.dining.economic_capacity,
        "economic_capacity",
        context,
        0.0,
        f32::INFINITY,
    )?;
    validate_range(
        &model.dining.max_satiation,
        "max_satiation",
        context,
        0.0,
        f32::INFINITY,
    )?;
    validate_range(
        &model.dining.eating_speed,
        "eating_speed",
        context,
        0.01,
        10.0,
    )?;

    // Validate personality ranges
    validate_range(
        &model.personality.patience_base,
        "patience_base",
        context,
        0.0,
        f32::INFINITY,
    )?;
    validate_range(
        &model.personality.decisiveness,
        "decisiveness",
        context,
        0.01,
        10.0,
    )?;
    validate_range(
        &model.personality.adaptiveness,
        "adaptiveness",
        context,
        0.0,
        1.0,
    )?;
    validate_range(
        &model.personality.confrontational,
        "confrontational",
        context,
        0.0,
        1.0,
    )?;

    Ok(())
}

/// Helper to validate a MinMax range
fn validate_range(
    range: &MinMax<f32>,
    field: &'static str,
    context: &str,
    absolute_min: f32,
    absolute_max: f32,
) -> ValidationResult {
    if range.min > range.max {
        return Err(ValidationError::InvalidValue {
            field,
            context: context.to_string(),
            reason: format!("min ({}) > max ({})", range.min, range.max),
        });
    }

    if range.min < absolute_min || range.max > absolute_max {
        return Err(ValidationError::InvalidValue {
            field,
            context: context.to_string(),
            reason: format!(
                "range [{}, {}] outside valid bounds [{}, {}]",
                range.min, range.max, absolute_min, absolute_max
            ),
        });
    }

    Ok(())
}
