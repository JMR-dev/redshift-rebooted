use redshift_rebooted::location::*;
use redshift_rebooted::types::*;

#[test]
fn test_geoclue2_location_provider_creation() {
    let provider = GeoClue2LocationProvider::new();
    assert_eq!(provider.name(), "geoclue2");
}

#[test]
fn test_geoclue2_location_provider_default() {
    let provider = GeoClue2LocationProvider::default();
    assert_eq!(provider.name(), "geoclue2");
}

#[test]
fn test_geoclue2_location_provider_init() {
    let mut provider = GeoClue2LocationProvider::new();
    assert!(
        provider.init().is_ok(),
        "GeoClue2 provider init should succeed"
    );
}

#[test]
fn test_geoclue2_location_provider_set_option_returns_error() {
    let mut provider = GeoClue2LocationProvider::new();
    provider.init().unwrap();

    // GeoClue2 provider doesn't accept any options
    let result = provider.set_option("some_key", "some_value");
    assert!(
        result.is_err(),
        "GeoClue2 provider should reject unknown options"
    );
}

#[test]
fn test_geoclue2_location_provider_trait_object() {
    let provider: Box<dyn LocationProvider> = Box::new(GeoClue2LocationProvider::new());
    assert_eq!(provider.name(), "geoclue2");
}

// Integration test - only runs if GeoClue2 is available and has location data
#[test]
#[ignore] // Use `cargo test -- --ignored` to run this
fn test_geoclue2_location_provider_integration() {
    let mut provider = GeoClue2LocationProvider::new();

    if provider.init().is_err() {
        eprintln!("GeoClue2 not available, skipping integration test");
        return;
    }

    if provider.start().is_err() {
        eprintln!("Could not start GeoClue2 provider, skipping");
        return;
    }

    // Wait for location
    std::thread::sleep(std::time::Duration::from_secs(10));

    match provider.get_location() {
        Ok(location) => {
            println!("Got location: {:.2}, {:.2}", location.lat, location.lon);
            assert!(
                location.lat >= MIN_LAT && location.lat <= MAX_LAT,
                "Latitude should be within valid range"
            );
            assert!(
                location.lon >= MIN_LON && location.lon <= MAX_LON,
                "Longitude should be within valid range"
            );
        }
        Err(e) => {
            eprintln!("No location available from GeoClue2: {}", e);
            eprintln!("This is expected if:");
            eprintln!("  - GeoClue2 doesn't have location sources (WiFi, GPS, etc.)");
            eprintln!("  - Location services are disabled");
            eprintln!("  - Redshift doesn't have permission to access location");
        }
    }
}

#[test]
#[ignore]
fn test_geoclue2_location_provider_lifecycle() {
    let mut provider = GeoClue2LocationProvider::new();

    assert!(provider.init().is_ok());
    assert!(provider.start().is_ok());

    // The provider should not panic when dropped
    drop(provider);
}

#[test]
fn test_geoclue2_location_provider_multiple_instances() {
    // Test that multiple instances can be created
    let mut provider1 = GeoClue2LocationProvider::new();
    let mut provider2 = GeoClue2LocationProvider::new();

    assert!(provider1.init().is_ok());
    assert!(provider2.init().is_ok());
}

#[test]
fn test_geoclue2_provider_get_location_before_start() {
    let mut provider = GeoClue2LocationProvider::new();
    provider.init().unwrap();

    // Should error if we try to get location before starting
    let result = provider.get_location();
    assert!(
        result.is_err(),
        "Should fail to get location before start()"
    );
}

#[test]
fn test_geoclue2_provider_print_help() {
    let provider = GeoClue2LocationProvider::new();
    // Should not panic
    provider.print_help();
}
