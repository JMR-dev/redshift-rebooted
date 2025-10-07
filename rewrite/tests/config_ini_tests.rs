use redshift_rebooted::config_ini::*;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_parse_brightness_single_value() {
    let (day, night) = parse_brightness_string("0.9").unwrap();
    assert_eq!(day, 0.9);
    assert_eq!(night, 0.9);
}

#[test]
fn test_parse_brightness_separate_values() {
    let (day, night) = parse_brightness_string("0.8:0.5").unwrap();
    assert_eq!(day, 0.8);
    assert_eq!(night, 0.5);
}

#[test]
fn test_parse_brightness_invalid() {
    assert!(parse_brightness_string("0.8:0.5:0.3").is_err());
    assert!(parse_brightness_string("invalid").is_err());
}

#[test]
fn test_parse_gamma_single_value() {
    let gamma = parse_gamma_string("0.8").unwrap();
    assert_eq!(gamma, [0.8, 0.8, 0.8]);
}

#[test]
fn test_parse_gamma_rgb_values() {
    let gamma = parse_gamma_string("0.8:0.7:0.9").unwrap();
    assert_eq!(gamma, [0.8, 0.7, 0.9]);
}

#[test]
fn test_parse_gamma_invalid() {
    assert!(parse_gamma_string("0.8:0.7").is_err()); // Only 2 values
    assert!(parse_gamma_string("invalid").is_err());
}

#[test]
fn test_load_full_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
temp-day=5700
temp-night=3500
fade=1
brightness-day=0.9
brightness-night=0.7
gamma=0.8:0.7:0.8
elevation-high=3
elevation-low=-6
dawn-time=6:00-7:45
dusk-time=18:35-20:15
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

    assert_eq!(config.temp_day, Some(5700));
    assert_eq!(config.temp_night, Some(3500));
    assert_eq!(config.fade, Some(true));
    assert_eq!(config.brightness_day, Some(0.9));
    assert_eq!(config.brightness_night, Some(0.7));
    assert_eq!(config.gamma_day, Some([0.8, 0.7, 0.8]));
    assert_eq!(config.gamma_night, Some([0.8, 0.7, 0.8]));
    assert_eq!(config.elevation_high, Some(3.0));
    assert_eq!(config.elevation_low, Some(-6.0));
    assert_eq!(config.manual_lat, Some(40.7));
    assert_eq!(config.manual_lon, Some(-74.0));
    assert_eq!(config.randr_screen, Some(0));

    // Check time ranges
    assert!(config.dawn_time.is_some());
    let dawn = config.dawn_time.unwrap();
    assert_eq!(dawn.start, 6 * 3600); // 6:00
    assert_eq!(dawn.end, 7 * 3600 + 45 * 60); // 7:45

    assert!(config.dusk_time.is_some());
    let dusk = config.dusk_time.unwrap();
    assert_eq!(dusk.start, 18 * 3600 + 35 * 60); // 18:35
    assert_eq!(dusk.end, 20 * 3600 + 15 * 60); // 20:15
}

#[test]
fn test_load_minimal_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
temp-day=6500
temp-night=4000
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert_eq!(config.temp_day, Some(6500));
    assert_eq!(config.temp_night, Some(4000));
    assert_eq!(config.brightness_day, None);
    assert_eq!(config.brightness_night, None);
}

#[test]
fn test_transition_alias() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
transition=0
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert_eq!(config.fade, Some(false));
}

#[test]
fn test_brightness_single_value_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
brightness=0.8
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert_eq!(config.brightness_day, Some(0.8));
    assert_eq!(config.brightness_night, Some(0.8));
}

#[test]
fn test_gamma_separate_day_night() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("redshift.conf");

    let config_content = r#"
[redshift]
gamma-day=0.8:0.7:0.8
gamma-night=0.6
"#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    let config = RedshiftConfig::load_from_file(&config_path).unwrap();

    assert_eq!(config.gamma_day, Some([0.8, 0.7, 0.8]));
    assert_eq!(config.gamma_night, Some([0.6, 0.6, 0.6]));
}

#[test]
fn test_get_manual_location() {
    let mut config = RedshiftConfig::default();
    assert!(config.get_manual_location().is_none());

    config.manual_lat = Some(40.7);
    config.manual_lon = Some(-74.0);

    let loc = config.get_manual_location().unwrap();
    assert_eq!(loc.lat, 40.7);
    assert_eq!(loc.lon, -74.0);
}

#[test]
fn test_config_search_paths() {
    let paths = RedshiftConfig::get_config_search_paths();

    // Should have at least the system-wide paths
    assert!(paths.len() >= 2);

    // Last two should be system paths
    let len = paths.len();
    assert!(paths[len - 2].to_str().unwrap().contains("/etc/redshift"));
    assert!(paths[len - 1].to_str().unwrap().contains("/etc/redshift.conf"));
}

#[test]
fn test_nonexistent_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.conf");

    let result = RedshiftConfig::load_from_file(&config_path);
    assert!(result.is_err());
}
