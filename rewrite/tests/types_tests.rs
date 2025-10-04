use redshift_rebooted::types::*;

#[test]
fn test_location_creation() {
    let loc = Location {
        lat: 40.7,
        lon: -74.0,
    };
    assert_eq!(loc.lat, 40.7);
    assert_eq!(loc.lon, -74.0);
}

#[test]
fn test_period_names() {
    assert_eq!(Period::None.name(), "None");
    assert_eq!(Period::Daytime.name(), "Daytime");
    assert_eq!(Period::Night.name(), "Night");
    assert_eq!(Period::Transition.name(), "Transition");
}

#[test]
fn test_color_setting_default() {
    let setting = ColorSetting::default();
    assert_eq!(setting.temperature, NEUTRAL_TEMP);
    assert_eq!(setting.gamma, [1.0, 1.0, 1.0]);
    assert_eq!(setting.brightness, 1.0);
}

#[test]
fn test_color_setting_custom() {
    let setting = ColorSetting {
        temperature: 3500,
        gamma: [0.9, 1.0, 1.1],
        brightness: 0.8,
    };
    assert_eq!(setting.temperature, 3500);
    assert_eq!(setting.gamma, [0.9, 1.0, 1.1]);
    assert_eq!(setting.brightness, 0.8);
}

#[test]
fn test_transition_scheme_default() {
    let scheme = TransitionScheme::default();
    assert_eq!(scheme.high, 3.0);
    assert_eq!(scheme.low, -6.0);
    assert!(!scheme.use_time);
    assert_eq!(scheme.day.temperature, NEUTRAL_TEMP);
    assert_eq!(scheme.night.temperature, 3500);
}

#[test]
fn test_time_range() {
    let range = TimeRange {
        start: 21600, // 6:00 AM
        end: 25200,   // 7:00 AM
    };
    assert_eq!(range.start, 21600);
    assert_eq!(range.end, 25200);
}

#[test]
fn test_bounds_constants() {
    assert_eq!(MIN_LAT, -90.0);
    assert_eq!(MAX_LAT, 90.0);
    assert_eq!(MIN_LON, -180.0);
    assert_eq!(MAX_LON, 180.0);
    assert_eq!(MIN_TEMP, 1000);
    assert_eq!(MAX_TEMP, 25000);
    assert_eq!(NEUTRAL_TEMP, 6500);
}

#[test]
fn test_period_equality() {
    assert_eq!(Period::Daytime, Period::Daytime);
    assert_ne!(Period::Daytime, Period::Night);
    assert_eq!(Period::Transition, Period::Transition);
}

#[test]
fn test_program_mode_variants() {
    let modes = [
        ProgramMode::Continual,
        ProgramMode::OneShot,
        ProgramMode::Print,
        ProgramMode::Reset,
        ProgramMode::Manual,
    ];

    assert_eq!(modes.len(), 5);
    assert_eq!(modes[0], ProgramMode::Continual);
    assert_eq!(modes[1], ProgramMode::OneShot);
}
