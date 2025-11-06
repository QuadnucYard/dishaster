//! Example program to test loading data from the assets directory

use dishaster_data::DataLoader;

#[test]
fn test_loading() -> Result<(), Box<dyn std::error::Error>> {
    let loader = DataLoader::new("../../assets/data")?;
    let data = loader.load_all_data()?;
    let registry = data.models;

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
        "✓ Loaded {} + {} trial statements",
        registry.trial.diner_speeches.len(),
        registry.trial.responses.len()
    );
    println!(
        "✓ Loaded {} QA ranks and {} AQ ranks",
        registry.trial.qa_ranks.len(),
        registry.trial.aq_ranks.len()
    );
    assert_eq!(
        registry.trial.diner_speeches.len(),
        registry.trial.qa_ranks.len(),
        "Mismatched number of diner speeches and QA ranks"
    );
    assert_eq!(
        registry.trial.responses.len(),
        registry.trial.aq_ranks.len(),
        "Mismatched number of responses and AQ ranks"
    );

    Ok(())
}
