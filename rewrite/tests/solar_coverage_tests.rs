/// Additional tests for solar module to improve coverage

// Import private functions for testing by using path
// Since these are in the main module, we need to test them indirectly

#[test]
fn test_solar_constants_are_defined() {
    // Test that the solar constants are accessible and correct
    use redshift_rebooted::solar::{
        SOLAR_ATM_REFRAC, SOLAR_ASTRO_TWILIGHT_ELEV, SOLAR_CIVIL_TWILIGHT_ELEV,
        SOLAR_DAYTIME_ELEV, SOLAR_NAUT_TWILIGHT_ELEV,
    };

    assert_eq!(SOLAR_ATM_REFRAC, 0.833);
    assert_eq!(SOLAR_ASTRO_TWILIGHT_ELEV, -18.0);
    assert_eq!(SOLAR_NAUT_TWILIGHT_ELEV, -12.0);
    assert_eq!(SOLAR_CIVIL_TWILIGHT_ELEV, -6.0);
    assert_eq!(SOLAR_DAYTIME_ELEV, 0.0 - SOLAR_ATM_REFRAC);
}

#[test]
fn test_solar_elevation_is_computed() {
    // Test that solar elevation can be computed without errors
    // Use equator at noon UTC for simplicity

    // March 20, 2024, ~12:00 UTC (approximate equinox)
    let date = 1710936000.0; // Unix timestamp
    let lat = 0.0; // Equator
    let lon = 0.0; // Prime meridian

    let elevation = redshift_rebooted::solar::solar_elevation(date, lat, lon);

    // Elevation should be in valid range
    assert!(elevation >= -90.0 && elevation <= 90.0, "Elevation should be in valid range");
}

#[test]
fn test_solar_elevation_at_midnight() {
    // Test solar elevation at midnight (lowest point)
    // Should be negative (sun below horizon)

    // March 20, 2024, ~00:00 UTC
    let date = 1710892800.0; // Unix timestamp
    let lat = 40.7;
    let lon = -74.0;

    let elevation = redshift_rebooted::solar::solar_elevation(date, lat, lon);

    // At midnight, sun should be below horizon
    assert!(elevation < 0.0, "Solar elevation at midnight should be negative");
}

#[test]
fn test_solar_table_fill_returns_all_times() {
    // Test that solar_table_fill returns a complete table
    use redshift_rebooted::solar::solar_table_fill;

    let date = 1710936000.0; // March 20, 2024, ~12:00 UTC
    let lat = 40.7;
    let lon = -74.0;

    let table = solar_table_fill(date, lat, lon);

    // Table should have 10 entries
    assert_eq!(table.len(), 10);

    // Noon and midnight should always be valid
    assert!(!table[0].is_nan(), "Noon should be valid"); // Noon is index 0
    assert!(!table[1].is_nan(), "Midnight should be valid"); // Midnight is index 1
}

#[test]
fn test_solar_table_fill_has_valid_noon() {
    // Test that solar_table_fill returns a valid noon timestamp
    use redshift_rebooted::solar::solar_table_fill;

    let date = 1710936000.0; // March 20, 2024
    let lat = 40.7;
    let lon = -74.0;

    let table = solar_table_fill(date, lat, lon);

    // Noon should always be valid
    let noon = table[0];
    assert!(!noon.is_nan(), "Noon should be valid");
    assert!(noon > 0.0, "Noon timestamp should be positive");
}

#[test]
fn test_solar_table_fill_polar_latitudes() {
    // Test solar_table_fill at polar latitudes where some events may not occur
    use redshift_rebooted::solar::solar_table_fill;

    // North pole in summer - midnight sun
    let date = 1718985600.0; // June 21, 2024 (summer solstice)
    let lat = 85.0; // Near north pole
    let lon = 0.0;

    let table = solar_table_fill(date, lat, lon);

    // Some twilight events may be NaN (sun never goes below certain elevations)
    // Just verify the function completes without panic
    assert_eq!(table.len(), 10);
}

#[test]
fn test_solar_table_fill_equator() {
    // Test solar_table_fill at the equator
    use redshift_rebooted::solar::solar_table_fill;

    let date = 1710936000.0; // March 20, 2024 (equinox)
    let lat = 0.0; // Equator
    let lon = 0.0;

    let table = solar_table_fill(date, lat, lon);

    // All events should be valid at equator on equinox
    for (i, &time) in table.iter().enumerate() {
        assert!(!time.is_nan(), "Event {} should be valid at equator", i);
    }
}

#[test]
fn test_solar_elevation_changes_over_time() {
    // Test that solar elevation changes over a 24-hour period
    use redshift_rebooted::solar::solar_elevation;

    let lat = 0.0; // Equator for simpler calculations
    let lon = 0.0;

    // Sample at different times
    let time1 = 1710892800.0;
    let time2 = 1710935400.0; // 12 hours later

    let el1 = solar_elevation(time1, lat, lon);
    let el2 = solar_elevation(time2, lat, lon);

    // Elevations should be different
    assert_ne!(el1, el2, "Solar elevation should change over time");
    assert!(el1 >= -90.0 && el1 <= 90.0, "Elevation 1 should be in valid range");
    assert!(el2 >= -90.0 && el2 <= 90.0, "Elevation 2 should be in valid range");
}

#[test]
fn test_solar_elevation_different_times_differ() {
    // Test that solar elevation differs at different times
    use redshift_rebooted::solar::solar_elevation;

    let lat = 40.7;
    let lon = -74.0;

    // Different times of day
    let time1 = 1710936000.0;
    let time2 = 1710946800.0; // Several hours later
    let time3 = 1710957600.0; // Several more hours later

    let el1 = solar_elevation(time1, lat, lon);
    let el2 = solar_elevation(time2, lat, lon);
    let el3 = solar_elevation(time3, lat, lon);

    // At least some values should differ
    assert!(el1 != el2 || el2 != el3, "Solar elevations should vary over time");
}

#[test]
fn test_solar_elevation_southern_hemisphere() {
    // Test solar elevation in southern hemisphere
    use redshift_rebooted::solar::solar_elevation;

    // Sydney, Australia (-33.9° S, 151.2° E)
    let date = 1710936000.0;
    let lat = -33.9;
    let lon = 151.2;

    let elevation = solar_elevation(date, lat, lon);

    // Should get a valid elevation
    assert!(elevation > -90.0 && elevation < 90.0, "Elevation should be in valid range");
}

#[test]
fn test_solar_elevation_negative_longitude() {
    // Test with negative longitude (western hemisphere)
    use redshift_rebooted::solar::solar_elevation;

    let date = 1710936000.0;
    let lat = 51.5; // London
    let lon = -0.1; // Slightly west of prime meridian

    let elevation = solar_elevation(date, lat, lon);

    // Should get a valid elevation
    assert!(elevation > -90.0 && elevation < 90.0, "Elevation should be in valid range");
}

#[test]
fn test_solar_table_fill_midnight_after_noon() {
    // Verify that midnight timestamp is 12 hours after noon
    use redshift_rebooted::solar::solar_table_fill;

    let date = 1710936000.0;
    let lat = 40.7;
    let lon = -74.0;

    let table = solar_table_fill(date, lat, lon);

    let noon = table[0];
    let midnight = table[1];

    // Midnight should be approximately 12 hours (43200 seconds) after noon
    let diff = midnight - noon;
    assert!((diff - 43200.0).abs() < 60.0, "Midnight should be ~12 hours after noon");
}

#[test]
fn test_solar_elevation_extreme_latitudes() {
    // Test solar elevation calculations at extreme latitudes
    use redshift_rebooted::solar::solar_elevation;

    let date = 1710936000.0;

    // Near north pole
    let el_north = solar_elevation(date, 89.0, 0.0);
    assert!(el_north > -90.0 && el_north < 90.0, "North pole elevation should be valid");

    // Near south pole
    let el_south = solar_elevation(date, -89.0, 0.0);
    assert!(el_south > -90.0 && el_south < 90.0, "South pole elevation should be valid");
}

#[test]
fn test_solar_elevation_full_day_cycle() {
    // Test that solar elevation follows expected pattern over 24 hours
    use redshift_rebooted::solar::solar_elevation;

    let lat = 40.7;
    let lon = -74.0;
    let start_date = 1710892800.0; // Midnight

    let mut max_elevation: f64 = -90.0;
    let mut min_elevation: f64 = 90.0;

    // Sample every 2 hours for 24 hours
    for hour in 0..12 {
        let date = start_date + (hour as f64 * 7200.0);
        let el = solar_elevation(date, lat, lon);

        max_elevation = max_elevation.max(el);
        min_elevation = min_elevation.min(el);
    }

    // Max should be positive (daytime), min should be negative (nighttime)
    assert!(max_elevation > 0.0, "Max elevation during day should be positive");
    assert!(min_elevation < 0.0, "Min elevation during night should be negative");
}
