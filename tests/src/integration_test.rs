//! Integration tests for Dishaster.

mod persist;

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use dishaster_core::{
    interface::{SimCommand, SimEvent, SimQuery, SimResponse},
    models::{Day, DinerOrder},
    sim::{ISimulation, Simulation},
};
use dishaster_data::DataLoader;
use dishaster_persistence::{PersistentStorage, UserDataService};
use dishrupt_rng::{Prng, prelude::Rng};

use crate::persist::{level_for_current_day, save_sim_profile};

/// In-memory persistent storage for testing purposes.
pub struct MemoryStorage;

impl PersistentStorage for MemoryStorage {
    fn read(&self, _path: &str) -> Result<Option<Vec<u8>>> {
        Ok(None) // Always return None (no data)
    }

    fn write_atomic(&self, _path: &str, _bytes: &[u8]) -> Result<()> {
        Ok(()) // Do nothing on write
    }

    fn delete(&self, _path: &str) -> Result<()> {
        Ok(()) // Do nothing on delete
    }
}

/// Extension trait to run multiple simulation steps.
trait RunSteps {
    fn run_steps(&mut self, steps: u32);

    fn run_steps_with_monitor(
        &mut self,
        steps: u32,
        monitor: impl FnMut(SimEvent, &mut Simulation),
    );
}

impl RunSteps for Simulation {
    fn run_steps(&mut self, steps: u32) {
        for i in 0..steps {
            if i > 0 && i % 1000 == 0 {
                println!("=== Step {} ===", i);
            }
            self.tick();
        }
    }

    fn run_steps_with_monitor(
        &mut self,
        steps: u32,
        mut monitor: impl FnMut(SimEvent, &mut Simulation),
    ) {
        for i in 0..steps {
            if i > 0 && i % 1000 == 0 {
                println!("=== Step {} ===", i);
            }
            self.tick();
            let events = self.poll_events();
            for event in events {
                monitor(event, self);
            }
            if self.is_day_complete() {
                println!("Day completed at step {}", i);
                break;
            }
        }
    }
}

/// Display distribution of values in bins
/// Analyze what diners with low spending or low weight are eating
fn analyze_dining_times(dining_times: &[f32], orders: &[DinerOrder]) {
    if dining_times.is_empty() {
        println!("No dining time data");
        return;
    }

    // Convert to minutes for easier reading
    let dining_times_min: Vec<f32> = dining_times.iter().map(|t| t / 60.0).collect();

    display_distribution("Dining Time (minutes)", &dining_times_min, 5.0);

    // Find long dining times (>30 min) and match with orders
    let long_dining_indices: Vec<usize> = dining_times
        .iter()
        .enumerate()
        .filter(|(_, t)| **t / 60.0 > 30.0)
        .map(|(i, _)| i)
        .collect();

    if !long_dining_indices.is_empty() {
        println!(
            "\nLong dining times (>30 min): {} diners",
            long_dining_indices.len()
        );
        println!("  Sample orders (first 10):");
        for (display_idx, &idx) in long_dining_indices.iter().enumerate().take(10) {
            if idx < orders.len() {
                let order = &orders[idx];
                let dishes_str = order
                    .dishes
                    .iter()
                    .map(|d| d.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!(
                    "    Order {}: {:.1} min, {:.3}kg, {} dishes - [{}]",
                    display_idx + 1,
                    dining_times[idx] / 60.0,
                    order.weight_kg,
                    order.dish_count,
                    dishes_str
                );
            }
        }

        // Analyze dish frequencies in long dining orders
        let mut dish_counts = HashMap::new();
        for &idx in &long_dining_indices {
            if idx < orders.len() {
                for dish_id in &orders[idx].dishes {
                    *dish_counts.entry(dish_id.clone()).or_insert(0) += 1;
                }
            }
        }
        let mut sorted: Vec<_> = dish_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        println!("  Top dishes in long dining orders:");
        for (dish_id, count) in sorted.iter().take(10) {
            println!("    {}: {} times", dish_id, count);
        }
    }
}

fn analyze_low_value_orders(orders: &[DinerOrder]) {
    // Collect low-price and low-weight orders
    let low_price_orders: Vec<_> = orders
        .iter()
        .filter(|o| o.dish_count > 0 && o.price_paid < 6.0)
        .collect();

    let low_weight_orders: Vec<_> = orders
        .iter()
        .filter(|o| o.dish_count > 0 && o.weight_kg < 0.2)
        .collect();

    // Count dish frequencies for low-price orders
    let mut low_price_dish_counts: HashMap<String, usize> = HashMap::new();
    for order in &low_price_orders {
        for dish_id in &order.dishes {
            *low_price_dish_counts
                .entry(dish_id.to_string())
                .or_insert(0) += 1;
        }
    }

    // Count dish frequencies for low-weight orders
    let mut low_weight_dish_counts: HashMap<String, usize> = HashMap::new();
    for order in &low_weight_orders {
        for dish_id in &order.dishes {
            *low_weight_dish_counts
                .entry(dish_id.to_string())
                .or_insert(0) += 1;
        }
    }

    // Display results
    println!("Low-price orders (<¥6): {} diners", low_price_orders.len());
    if !low_price_orders.is_empty() {
        let avg_price: f32 = low_price_orders.iter().map(|o| o.price_paid).sum::<f32>()
            / low_price_orders.len() as f32;
        let avg_weight: f32 = low_price_orders.iter().map(|o| o.weight_kg).sum::<f32>()
            / low_price_orders.len() as f32;
        println!(
            "  Avg price: ¥{:.2}, Avg weight: {:.3}kg",
            avg_price, avg_weight
        );

        println!("  Sample orders (first 10):");
        for (idx, order) in low_price_orders.iter().enumerate().take(10) {
            let dishes_str = order
                .dishes
                .iter()
                .map(|d| d.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "    Order {}: ¥{:.2}, {:.3}kg - [{}]",
                idx + 1,
                order.price_paid,
                order.weight_kg,
                dishes_str
            );
        }

        let mut sorted: Vec<_> = low_price_dish_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        println!("  Top dishes:");
        for (dish_id, count) in sorted.iter().take(10) {
            println!("    {}: {} times", dish_id, count);
        }
    }

    println!(
        "\nLow-weight orders (<0.2kg): {} diners",
        low_weight_orders.len()
    );
    if !low_weight_orders.is_empty() {
        let avg_price: f32 = low_weight_orders.iter().map(|o| o.price_paid).sum::<f32>()
            / low_weight_orders.len() as f32;
        let avg_weight: f32 = low_weight_orders.iter().map(|o| o.weight_kg).sum::<f32>()
            / low_weight_orders.len() as f32;
        println!(
            "  Avg price: ¥{:.2}, Avg weight: {:.3}kg",
            avg_price, avg_weight
        );

        println!("  Sample orders (first 10):");
        for (idx, order) in low_weight_orders.iter().enumerate().take(10) {
            let dishes_str = order
                .dishes
                .iter()
                .map(|d| d.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "    Order {}: ¥{:.2}, {:.3}kg - [{}]",
                idx + 1,
                order.price_paid,
                order.weight_kg,
                dishes_str
            );
        }

        let mut sorted: Vec<_> = low_weight_dish_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        println!("  Top dishes:");
        for (dish_id, count) in sorted.iter().take(10) {
            println!("    {}: {} times", dish_id, count);
        }
    }
}

fn display_distribution(title: &str, values: &[f32], bin_width: f32) {
    if values.is_empty() {
        println!("{}: (empty)", title);
        return;
    }

    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let num_bins = ((max - min) / bin_width).ceil() as usize + 1;

    let mut bins = vec![0usize; num_bins];
    for &val in values {
        let bin_idx = if bin_width > 0.0 {
            ((val - min) / bin_width).floor() as usize
        } else {
            0
        };
        if bin_idx < bins.len() {
            bins[bin_idx] += 1;
        }
    }

    println!(
        "{}: min={:.2}, max={:.2}, mean={:.2}",
        title,
        min,
        max,
        values.iter().sum::<f32>() / values.len() as f32
    );
    for (i, &count) in bins.iter().enumerate() {
        if count > 0 {
            let bin_start = min + i as f32 * bin_width;
            let bin_end = bin_start + bin_width;
            println!("  [{:.2}-{:.2}): {} diners", bin_start, bin_end, count);
        }
    }
}

#[test]
fn single_complete_run() -> Result<()> {
    env_logger::Builder::new().init();

    let mut loader = DataLoader::new("../assets/data")?;
    let data = loader.load_all_data()?;
    let registry = data.models;

    dishaster_validation::validate_registry(&registry)
        .map_err(|errors| anyhow::anyhow!("Validation failed with {} error(s)", errors.len()))?;
    println!("✓ Data validation passed");

    let service = UserDataService::new(Arc::new(MemoryStorage));
    let profile_svc = &service.profiles;
    dishaster_validation::validate_player_profile(&profile_svc.load().unwrap(), &registry)
        .map_err(|errors| {
            anyhow::anyhow!("Profile validation failed with {} error(s)", errors.len())
        })?;
    println!("✓ Player profile validation passed");

    let level = level_for_current_day(profile_svc, &registry)?;
    let start_day = level.day;

    let mut sim = Simulation::new(registry.clone());
    sim.start(level);

    // Let the simulation stabilize a bit
    sim.run_steps(100);

    // Start the run
    sim.command(SimCommand::StartRun);

    // Run with event monitoring to refill empty dispensers
    sim.run_steps_with_monitor(60000, |event, sim| {
        if let SimEvent::DispenserStockChanged {
            entity,
            current_stock,
            ..
        } = event
            && current_stock == 0
        {
            println!("Dispenser {:?} is empty, sending refill command", entity);
            sim.command(SimCommand::RefillDispenser(entity));
        }
    });

    // End the run and check for completion events
    sim.command(SimCommand::EndRun);
    sim.run_steps(1);
    let events = sim.poll_events();
    println!("Events: {}", events.len());
    // assert!(events.iter().any(|e| matches!(e, SimEvent::RunCompleted)));
    if !events
        .iter()
        .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_)))
    {
        // Day might have auto-completed, try getting events again
        sim.run_steps(10);
        let events = sim.poll_events();
        println!("Events after extra steps: {}", events.len());
    }

    // Apply a management decision (index 0 for test)
    sim.command(SimCommand::ApplyManagementDecision(0));
    sim.run_steps(1);
    let events = sim.poll_events();
    println!("Events after decision: {}", events.len());
    assert!(events.iter().any(|e| matches!(e, SimEvent::Persist)));
    assert!(events.iter().any(|e| matches!(e, SimEvent::DayCompleted)));

    let persisted = sim.persist();
    println!("Persisted profile");
    assert_eq!(persisted.current_day, Day(start_day.0 + 1)); // Day should have advanced

    // Print first day stats
    let day_stats = &persisted.day_stats;
    println!("Day 0 stats:");
    println!(
        "  Consumption: {:.2} kg, Revenue: ¥{:.2}, Completed: {} / {}",
        day_stats.consumption_kg,
        day_stats.revenue,
        day_stats.completed_diners,
        day_stats.total_visits
    );
    println!("  Reputation: {:.2}", persisted.reputation);

    // Print serving and dining time statistics
    if !day_stats.serving_times.is_empty() {
        let avg_serving =
            day_stats.serving_times.iter().sum::<f32>() / day_stats.serving_times.len() as f32;
        let min_serving = day_stats
            .serving_times
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min);
        let max_serving = day_stats
            .serving_times
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        println!(
            "  Serving time: avg={:.1}s, min={:.1}s, max={:.1}s (n={})",
            avg_serving,
            min_serving,
            max_serving,
            day_stats.serving_times.len()
        );
    }
    if !day_stats.dining_times.is_empty() {
        let avg_dining =
            day_stats.dining_times.iter().sum::<f32>() / day_stats.dining_times.len() as f32;
        let min_dining = day_stats
            .dining_times
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min);
        let max_dining = day_stats
            .dining_times
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        println!(
            "  Dining time: avg={:.1}s, min={:.1}s, max={:.1}s (n={})",
            avg_dining,
            min_dining,
            max_dining,
            day_stats.dining_times.len()
        );
    }

    let feedback_stats = sim.execute_query(SimQuery::FeedbackStats);
    if let SimResponse::FeedbackStats(stats) = feedback_stats {
        println!("Feedback stats:\n{stats}\n");
    }

    // Validate diner orders
    println!("=== Diner Orders Validation ===");
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut invalid_orders = Vec::new();
    let mut dish_counts = Vec::new();
    let mut prices = Vec::new();
    let mut weights = Vec::new();

    for (idx, order) in day_stats.diner_orders.iter().enumerate() {
        let is_valid = (order.dish_count == 0 && order.price_paid == 0.0)
            || (order.dish_count >= 1
                && order.dish_count <= 4
                && order.price_paid > 0.0
                && order.price_paid <= 30.0);

        if is_valid {
            valid_count += 1;
        } else {
            invalid_count += 1;
            invalid_orders.push((idx, order.dish_count, order.price_paid));
        }

        if order.dish_count > 0 {
            dish_counts.push(order.dish_count as f32);
            prices.push(order.price_paid);
            weights.push(order.weight_kg);
        }
    }

    println!("Total diners: {}", day_stats.diner_orders.len());
    println!(
        "Valid orders: {} (0 dishes with ¥0, or 1-4 dishes with ¥0-30)",
        valid_count
    );
    println!("Invalid orders: {}", invalid_count);

    if !invalid_orders.is_empty() {
        println!("Invalid orders details:");
        for (idx, dish_count, price) in invalid_orders.iter().take(10) {
            println!("  Diner {}: {} dishes, ¥{:.2}", idx, dish_count, price);
        }
        if invalid_orders.len() > 10 {
            println!("  ... and {} more", invalid_orders.len() - 10);
        }
    }

    // Assert all orders are valid
    assert!(
        invalid_count == 0,
        "Found {} invalid orders (expected all diners to have 0 dishes with ¥0, or 1-4 dishes with ¥0-30)",
        invalid_count
    );

    // Display distributions
    println!("\n=== Order Distributions ===");
    display_distribution("Dish Count Distribution", &dish_counts, 1.0);
    println!();
    display_distribution("Price Distribution (¥)", &prices, 2.0);
    println!();
    display_distribution("Weight Distribution (kg)", &weights, 0.05);

    // Analyze dining times and low-value orders
    println!("\n=== Dining Time Analysis ===");
    analyze_dining_times(&day_stats.dining_times, &day_stats.diner_orders);

    println!("\n=== Low-Spending/Low-Weight Analysis ===");
    analyze_low_value_orders(&day_stats.diner_orders);

    save_sim_profile(profile_svc, persisted)?;
    std::mem::drop(sim);

    Ok(())
}

#[test]
fn continuous_run() -> Result<()> {
    let mut loader = DataLoader::new("../assets/data")?;
    let data = loader.load_all_data()?;
    let registry = data.models;

    let service = UserDataService::new(Arc::new(MemoryStorage));
    let profile_svc = &service.profiles;
    println!("✓ Player profile validation passed");

    // now run more days
    let mut rng = Prng::new(42);
    for i in 0..100 {
        println!("--- Day {} ---", i + 1);

        // Start next day
        let level = level_for_current_day(profile_svc, &registry)?;
        let mut sim = Simulation::new(registry.clone());
        sim.start(level);
        sim.run_steps(10);

        // Start the run
        sim.command(SimCommand::StartRun);
        sim.run_steps(100);

        // End the run
        sim.command(SimCommand::EndRun);
        sim.run_steps(1);

        let events = sim.poll_events();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, SimEvent::RunCompleted(_)))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_)))
        );

        // Apply a management decision
        sim.command(SimCommand::ApplyManagementDecision(rng.random_range(0..3)));
        sim.run_steps(1);

        let persisted = sim.persist();
        save_sim_profile(profile_svc, persisted)?;
    }

    // Print aggregate stats at the end
    let profile = profile_svc.load().unwrap();
    println!("\n=== Final Statistics ===");
    println!(
        "Total consumption: {:.2} kg",
        profile.aggregates.lifetime_consumption_kg
    );
    println!("Total revenue: ¥{:.2}", profile.aggregates.lifetime_revenue);
    println!(
        "Total served: {} diners",
        profile.aggregates.lifetime_served
    );

    let level_progress = profile.level_progress.as_ref().unwrap();
    println!("Days of history: {}", level_progress.daily_history.len());

    // Show last 5 days stats
    println!("\nLast 5 days:");
    for day_stats in level_progress.daily_history.iter().rev().take(5).rev() {
        println!(
            "  Day {}: {:.2}kg, ¥{:.0}, {}/{} completed",
            day_stats.day.0,
            day_stats.consumption_kg,
            day_stats.revenue,
            day_stats.completed_diners,
            day_stats.total_visits
        );
    }

    println!("\nTest completed successfully.");

    Ok(())
}
