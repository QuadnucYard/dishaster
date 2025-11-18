//! Example program to test loading data from the assets directory

use dishaster_data::DataLoader;

#[test]
fn test_loading() -> Result<(), Box<dyn std::error::Error>> {
    let mut loader = DataLoader::new("../../assets/data")?;
    let data = loader.load_all_data()?;
    let registry = data.models;

    println!("Reputation config: {:#?}", registry.reputation_config);
    println!("Ordering config: {:#?}", registry.ordering_config);
    println!("Decision config: {:#?}", registry.decision_config);

    println!("✓ Loaded {} levels", registry.levels.len());
    println!("✓ Loaded {} canteens", registry.canteens.len());
    println!("✓ Loaded {} dishes", registry.dishes.len());
    println!(
        "✓ Loaded {} window services",
        registry.window_services.len()
    );
    println!("✓ Loaded {} tables", registry.tables.len());
    println!("✓ Loaded {} dispensers", registry.dispensers.len());
    println!("✓ Loaded {} collectors", registry.collectors.len());

    println!(
        "✓ Loaded {} management decisions",
        registry.mgmt_decisions.len()
    );
    println!(
        "✓ Loaded {} management incidents",
        registry.mgmt_incidents.len()
    );

    println!(
        "✓ Loaded {} + {} trial statements",
        registry.trial.diner_speeches.len(),
        registry.trial.responses.len()
    );
    println!(
        "✓ Loaded {} QA ranks and {} AQ ranks",
        registry.trial.qa_ranks.len(),
        registry.trial.aq_ranks.len()
    );

    // Validate all loaded data
    dishaster_validation::validate_registry(&registry)
        .map_err(|errors| anyhow::anyhow!("Validation failed with {} error(s)", errors.len()))?;
    println!("✓ Data validation passed");

    Ok(())
}
