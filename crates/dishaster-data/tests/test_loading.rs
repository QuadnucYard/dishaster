//! Example program to test loading data from the assets directory

use dishaster_data::DataLoader;

#[test]
fn test_loading() -> Result<(), Box<dyn std::error::Error>> {
    let loader = DataLoader::new("../../assets/data")?;
    let registry = loader.load_all_data()?;

    println!("✓ Loaded {} canteens", registry.canteens.len());
    println!("✓ Loaded {} dishes", registry.dishes.len());
    println!(
        "✓ Loaded {} window services",
        registry.window_services.len()
    );
    println!("✓ Loaded {} tables", registry.tables.len());
    println!("✓ Loaded {} dispensers", registry.dispensers.len());
    println!("✓ Loaded {} collectors", registry.collectors.len());

    Ok(())
}
