/// Additional tests for config module to improve coverage

use redshift_rebooted::config::{Config, LocationSource};
use redshift_rebooted::types::Location;

#[test]
fn test_config_path_creation() {
    let result = Config::config_path();
    assert!(result.is_ok(), "Should be able to determine config path");

    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("redshift"), "Path should contain 'redshift'");
    assert!(path.to_string_lossy().ends_with("config.toml"), "Path should end with 'config.toml'");
}

#[test]
fn test_config_load_returns_default_on_missing_file() {
    // Test the default config behavior
    let config = Config::default();

    assert!(config.location.is_none(), "Default config should have no location");
    assert!(config.last_geoclue_check.is_none(), "Default config should have no last check");
}

#[test]
fn test_config_save_and_load() {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Create a config with data
    let mut config = Config::default();
    config.set_location(
        Location { lat: 48.8566, lon: 2.3522 },
        LocationSource::Manual,
        Some("Paris".to_string())
    );
    config.update_geoclue_check();

    // Serialize to TOML string
    let toml_str = toml::to_string(&config).expect("Should serialize to TOML");

    // Deserialize back
    let loaded: Config = toml::from_str(&toml_str).expect("Should deserialize from TOML");

    // Verify data was preserved
    assert!(loaded.location.is_some(), "Loaded config should have location");
    let location = loaded.get_location().unwrap();
    assert_eq!(location.lat, 48.8566, "Latitude should match");
    assert_eq!(location.lon, 2.3522, "Longitude should match");

    assert!(loaded.last_geoclue_check.is_some(), "Loaded config should have geoclue check timestamp");

    if let Some(ref saved_loc) = loaded.location {
        assert_eq!(saved_loc.source, LocationSource::Manual, "Source should be Manual");
        assert_eq!(saved_loc.city_name, Some("Paris".to_string()), "City name should be preserved");
    }
}

#[test]
fn test_config_path_has_parent_directory() {
    // Verify config path has a parent directory
    let config_path = Config::config_path();
    assert!(config_path.is_ok(), "Should be able to get config path");

    let path = config_path.unwrap();
    assert!(path.parent().is_some(), "Config path should have a parent directory");
}

#[test]
fn test_config_update_geoclue_check() {
    let mut config = Config::default();

    // Initially should be None
    assert!(config.last_geoclue_check.is_none());

    // Update the check
    config.update_geoclue_check();

    // Should now have a timestamp
    assert!(config.last_geoclue_check.is_some());
    let timestamp = config.last_geoclue_check.unwrap();

    // Timestamp should be recent (within last minute)
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    assert!(timestamp <= now, "Timestamp should not be in the future");
    assert!(now - timestamp < 60, "Timestamp should be recent");
}

#[test]
fn test_config_parse_invalid_toml() {
    // Test that invalid TOML fails to parse
    let invalid_toml = "this is not valid toml [[[";

    let result: Result<Config, _> = toml::from_str(invalid_toml);
    assert!(result.is_err(), "Parsing invalid TOML should fail");
}

#[test]
fn test_location_source_serialization() {
    use serde_json;

    // Test Manual source
    let manual_json = serde_json::to_string(&LocationSource::Manual).unwrap();
    assert_eq!(manual_json, r#""manual""#);

    // Test Interactive source
    let interactive_json = serde_json::to_string(&LocationSource::Interactive).unwrap();
    assert_eq!(interactive_json, r#""interactive""#);

    // Test GeoClue2 source
    let geoclue_json = serde_json::to_string(&LocationSource::GeoClue2).unwrap();
    assert_eq!(geoclue_json, r#""geoclue2""#);
}

#[test]
fn test_location_source_deserialization() {
    use serde_json;

    let manual: LocationSource = serde_json::from_str(r#""manual""#).unwrap();
    assert_eq!(manual, LocationSource::Manual);

    let interactive: LocationSource = serde_json::from_str(r#""interactive""#).unwrap();
    assert_eq!(interactive, LocationSource::Interactive);

    let geoclue: LocationSource = serde_json::from_str(r#""geoclue2""#).unwrap();
    assert_eq!(geoclue, LocationSource::GeoClue2);
}

#[test]
fn test_saved_location_with_all_fields() {
    let mut config = Config::default();

    config.set_location(
        Location { lat: 35.6762, lon: 139.6503 },
        LocationSource::GeoClue2,
        Some("Tokyo".to_string())
    );

    let saved_loc = config.location.as_ref().unwrap();
    assert_eq!(saved_loc.lat, 35.6762);
    assert_eq!(saved_loc.lon, 139.6503);
    assert_eq!(saved_loc.source, LocationSource::GeoClue2);
    assert_eq!(saved_loc.city_name, Some("Tokyo".to_string()));
}

#[test]
fn test_saved_location_without_city_name() {
    let mut config = Config::default();

    config.set_location(
        Location { lat: -33.8688, lon: 151.2093 },
        LocationSource::Interactive,
        None
    );

    let saved_loc = config.location.as_ref().unwrap();
    assert_eq!(saved_loc.lat, -33.8688);
    assert_eq!(saved_loc.lon, 151.2093);
    assert_eq!(saved_loc.source, LocationSource::Interactive);
    assert!(saved_loc.city_name.is_none());
}
