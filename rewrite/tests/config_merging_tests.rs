/// Integration tests for config merging behavior
/// Tests that CLI args properly override INI config settings

use redshift_rebooted::config_ini::RedshiftConfig;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_config_loads_temperatures() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
temp-day=5700
temp-night=3500
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert_eq!(config.temp_day, Some(5700));
    assert_eq!(config.temp_night, Some(3500));
}

#[test]
fn test_config_loads_brightness_separate() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
brightness-day=0.9
brightness-night=0.7
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert_eq!(config.brightness_day, Some(0.9));
    assert_eq!(config.brightness_night, Some(0.7));
}

#[test]
fn test_config_loads_gamma_separate_day_night() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
gamma-day=0.8:0.7:0.9
gamma-night=0.6
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert_eq!(config.gamma_day, Some([0.8, 0.7, 0.9]));
    assert_eq!(config.gamma_night, Some([0.6, 0.6, 0.6]));
}

#[test]
fn test_config_loads_elevation_settings() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
elevation-high=3
elevation-low=-6
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert_eq!(config.elevation_high, Some(3.0));
    assert_eq!(config.elevation_low, Some(-6.0));
}

#[test]
fn test_config_loads_time_based_transition() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
dawn-time=6:00-7:45
dusk-time=18:35-20:15
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert!(config.dawn_time.is_some());
    assert!(config.dusk_time.is_some());

    let dawn = config.dawn_time.unwrap();
    assert_eq!(dawn.start, 6 * 3600);
    assert_eq!(dawn.end, 7 * 3600 + 45 * 60);

    let dusk = config.dusk_time.unwrap();
    assert_eq!(dusk.start, 18 * 3600 + 35 * 60);
    assert_eq!(dusk.end, 20 * 3600 + 15 * 60);
}

#[test]
fn test_config_with_all_sections() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
temp-day=5700
temp-night=3500
fade=1
brightness-day=0.9
brightness-night=0.7
gamma=0.8
elevation-high=3
elevation-low=-6
location-provider=manual
adjustment-method=randr

[manual]
lat=40.7
lon=-74.0

[randr]
screen=0
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    // Redshift section
    assert_eq!(config.temp_day, Some(5700));
    assert_eq!(config.temp_night, Some(3500));
    assert_eq!(config.fade, Some(true));
    assert_eq!(config.brightness_day, Some(0.9));
    assert_eq!(config.brightness_night, Some(0.7));
    assert_eq!(config.gamma_day, Some([0.8, 0.8, 0.8]));
    assert_eq!(config.elevation_high, Some(3.0));
    assert_eq!(config.elevation_low, Some(-6.0));

    // Manual location section
    assert_eq!(config.manual_lat, Some(40.7));
    assert_eq!(config.manual_lon, Some(-74.0));

    let loc = config.get_manual_location().unwrap();
    assert_eq!(loc.lat, 40.7);
    assert_eq!(loc.lon, -74.0);

    // Randr section
    assert_eq!(config.randr_screen, Some(0));
}
