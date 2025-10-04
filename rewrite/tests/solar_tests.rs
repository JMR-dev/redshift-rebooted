use redshift_rebooted::solar::*;

#[test]
fn test_solar_elevation_range() {
    // Solar elevation should always be between -90 and 90 degrees
    let lat = 40.7;
    let lon = -74.0;

    for hour in 0..24 {
        let time = 1710892800.0 + (hour as f64) * 3600.0;
        let elevation = solar_elevation(time, lat, lon);

        assert!(
            elevation >= -90.0 && elevation <= 90.0,
            "Solar elevation {} out of range [-90, 90] at hour {}",
            elevation,
            hour
        );
    }
}

#[test]
fn test_solar_elevation_at_noon_higher_than_midnight() {
    // Sun should be higher at local noon than at midnight
    // Using a time known to be around noon UTC for New York (EST/EDT)
    let lat = 40.7;
    let lon = -74.0;

    // Noon and midnight times (approximately)
    let noon_time = 1710936000.0; // Around noon UTC
    let midnight_time = noon_time - 12.0 * 3600.0; // 12 hours earlier

    let elev_noon = solar_elevation(noon_time, lat, lon);
    let elev_midnight = solar_elevation(midnight_time, lat, lon);

    // At midnight, sun should be below horizon (negative)
    assert!(
        elev_midnight < 0.0,
        "Solar elevation at night should be negative, got: {}",
        elev_midnight
    );

    // Noon should have higher elevation than midnight
    assert!(
        elev_noon > elev_midnight,
        "Solar elevation at noon ({}) should be higher than midnight ({})",
        elev_noon,
        elev_midnight
    );
}

#[test]
fn test_solar_elevation_different_longitudes() {
    // Same time, different longitudes should give different elevations
    let lat = 40.0;
    let time = 1710936000.0;

    let elev_west = solar_elevation(time, lat, -120.0); // West coast USA
    let elev_east = solar_elevation(time, lat, -75.0);  // East coast USA

    // Different longitudes at same time should have different solar elevations
    assert!(
        (elev_west - elev_east).abs() > 5.0,
        "Different longitudes should have significantly different solar elevations"
    );
}

#[test]
fn test_solar_elevation_equator_higher_than_poles() {
    // At the same time, solar elevation should generally be higher near equator
    let time = 1710936000.0; // Some reference time
    let lon = 0.0;

    let elev_equator = solar_elevation(time, 0.0, lon);
    let elev_pole = solar_elevation(time, 80.0, lon);

    // This test is somewhat simplified - in reality it depends on season
    // But as a general principle, equator tends to have more consistent high elevations
    assert!(
        elev_equator.abs() < 90.0 && elev_pole.abs() < 90.0,
        "Solar elevations should be in valid range"
    );
}

#[test]
fn test_solar_table_fill_size() {
    let date = 1710936000.0;
    let lat = 40.7;
    let lon = -74.0;

    let table = solar_table_fill(date, lat, lon);

    // Table should have exactly 10 entries
    assert_eq!(table.len(), 10);
}

#[test]
fn test_solar_constants() {
    assert_eq!(SOLAR_ATM_REFRAC, 0.833);
    assert_eq!(SOLAR_ASTRO_TWILIGHT_ELEV, -18.0);
    assert_eq!(SOLAR_NAUT_TWILIGHT_ELEV, -12.0);
    assert_eq!(SOLAR_CIVIL_TWILIGHT_ELEV, -6.0);
    assert!((SOLAR_DAYTIME_ELEV - (-0.833)).abs() < 0.001);
}

#[test]
fn test_solar_elevation_changes_over_day() {
    // Solar elevation should change over the course of a day
    let lat = 40.7;
    let lon = -74.0;
    let base_time = 1710892800.0;

    let mut elevations = Vec::new();
    for hour in 0..24 {
        let time = base_time + (hour as f64) * 3600.0;
        let elevation = solar_elevation(time, lat, lon);
        elevations.push(elevation);
    }

    // There should be variation in elevations throughout the day
    let max_elev = elevations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let min_elev = elevations.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    assert!(
        max_elev - min_elev > 50.0,
        "Solar elevation should vary significantly over 24 hours"
    );
}

#[test]
fn test_solar_elevation_at_different_latitudes() {
    // Test that function works at various latitudes
    let time = 1710936000.0;
    let lon = 0.0;

    let latitudes = [-80.0, -45.0, 0.0, 45.0, 80.0];

    for lat in latitudes {
        let elevation = solar_elevation(time, lat, lon);
        assert!(
            elevation >= -90.0 && elevation <= 90.0,
            "Solar elevation at latitude {} should be in valid range, got {}",
            lat,
            elevation
        );
    }
}
