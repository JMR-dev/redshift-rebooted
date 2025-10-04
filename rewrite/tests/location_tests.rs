use redshift_rebooted::location::*;
use redshift_rebooted::types::Location;

#[test]
fn test_manual_location_provider_new() {
    let provider = ManualLocationProvider::new();
    assert_eq!(provider.name(), "manual");
}

#[test]
fn test_manual_location_provider_with_location() {
    let mut provider = ManualLocationProvider::with_location(40.7, -74.0);

    provider.init().expect("Init should succeed");
    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 40.7);
    assert_eq!(location.lon, -74.0);
}

#[test]
fn test_manual_location_provider_set_lat() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("lat", "51.5")
        .expect("Setting lat should succeed");
    provider
        .set_option("lon", "0.0")
        .expect("Setting lon should succeed");
    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 51.5);
    assert_eq!(location.lon, 0.0);
}

#[test]
fn test_manual_location_provider_set_lon() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("lat", "40.7")
        .expect("Setting lat should succeed");
    provider
        .set_option("lon", "-74.0")
        .expect("Setting lon should succeed");
    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 40.7);
    assert_eq!(location.lon, -74.0);
}

#[test]
fn test_manual_location_provider_case_insensitive() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("LAT", "40.7")
        .expect("LAT should work");
    provider
        .set_option("Lon", "-74.0")
        .expect("Lon should work");
    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 40.7);
    assert_eq!(location.lon, -74.0);
}

#[test]
fn test_manual_location_provider_start_without_location() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");

    // Starting without setting location should fail
    let result = provider.start();
    assert!(
        result.is_err(),
        "Start should fail without location being set"
    );
    assert!(result
        .unwrap_err()
        .contains("Latitude and longitude must be set"));
}

#[test]
fn test_manual_location_provider_partial_location() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("lat", "40.7")
        .expect("Setting lat should succeed");

    // With only lat set, lon defaults to 0.0, so start should succeed
    provider.start().expect("Start should succeed with lat set (lon defaults to 0.0)");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 40.7);
    assert_eq!(location.lon, 0.0, "Longitude should default to 0.0");
}

#[test]
fn test_manual_location_provider_invalid_option() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");

    let result = provider.set_option("invalid", "42.0");
    assert!(result.is_err(), "Invalid option should return error");
    assert!(result.unwrap_err().contains("Unknown method parameter"));
}

#[test]
fn test_manual_location_provider_invalid_value() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");

    let result = provider.set_option("lat", "not_a_number");
    assert!(result.is_err(), "Invalid value should return error");
    assert!(result.unwrap_err().contains("Malformed argument"));
}

#[test]
fn test_manual_location_provider_negative_coordinates() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("lat", "-33.9")
        .expect("Negative lat should work");
    provider
        .set_option("lon", "151.2")
        .expect("Positive lon should work");
    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, -33.9);
    assert_eq!(location.lon, 151.2);
}

#[test]
fn test_manual_location_provider_zero_coordinates() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("lat", "0.0")
        .expect("Zero lat should work");
    provider
        .set_option("lon", "0.0")
        .expect("Zero lon should work");
    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 0.0);
    assert_eq!(location.lon, 0.0);
}

#[test]
fn test_manual_location_provider_extreme_coordinates() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("lat", "89.9")
        .expect("Near-pole lat should work");
    provider
        .set_option("lon", "-179.9")
        .expect("Near-antimeridian lon should work");
    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 89.9);
    assert_eq!(location.lon, -179.9);
}

#[test]
fn test_manual_location_provider_decimal_precision() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("lat", "40.7128")
        .expect("Precise lat should work");
    provider
        .set_option("lon", "-74.0060")
        .expect("Precise lon should work");
    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert!((location.lat - 40.7128).abs() < 0.0001);
    assert!((location.lon - (-74.0060)).abs() < 0.0001);
}

#[test]
fn test_manual_location_provider_print_help() {
    let provider = ManualLocationProvider::new();

    // This should not panic
    provider.print_help();
}

#[test]
fn test_manual_location_provider_overwrite_location() {
    let mut provider = ManualLocationProvider::new();

    provider.init().expect("Init should succeed");
    provider
        .set_option("lat", "40.7")
        .expect("Setting initial lat should succeed");
    provider
        .set_option("lon", "-74.0")
        .expect("Setting initial lon should succeed");

    // Overwrite with new values
    provider
        .set_option("lat", "51.5")
        .expect("Overwriting lat should succeed");
    provider
        .set_option("lon", "0.0")
        .expect("Overwriting lon should succeed");

    provider.start().expect("Start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 51.5);
    assert_eq!(location.lon, 0.0);
}

#[test]
fn test_manual_location_provider_get_before_start() {
    let mut provider = ManualLocationProvider::with_location(40.7, -74.0);

    provider.init().expect("Init should succeed");

    // Get location should work even before start
    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 40.7);
    assert_eq!(location.lon, -74.0);
}

#[test]
fn test_manual_location_provider_multiple_starts() {
    let mut provider = ManualLocationProvider::with_location(40.7, -74.0);

    provider.init().expect("Init should succeed");
    provider.start().expect("First start should succeed");
    provider.start().expect("Second start should succeed");

    let location = provider.get_location().expect("Should get location");
    assert_eq!(location.lat, 40.7);
    assert_eq!(location.lon, -74.0);
}

#[test]
fn test_location_struct_copy() {
    let loc1 = Location {
        lat: 40.7,
        lon: -74.0,
    };
    let loc2 = loc1; // Should copy, not move

    // Both should be usable
    assert_eq!(loc1.lat, 40.7);
    assert_eq!(loc2.lat, 40.7);
}

#[test]
fn test_manual_location_default() {
    let provider1 = ManualLocationProvider::new();
    let provider2 = ManualLocationProvider::default();

    assert_eq!(provider1.name(), provider2.name());
}
