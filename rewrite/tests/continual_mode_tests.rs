/// Tests for continual mode functionality
/// These tests verify the main event loop logic without actually running the infinite loop

use redshift_rebooted::types::{ColorSetting, TransitionScheme, NEUTRAL_TEMP};

/* Helper function to calculate transition progress from elevation.
   This is the same logic used in main.rs */
fn get_transition_progress_from_elevation(scheme: &TransitionScheme, elevation: f64) -> f64 {
    if elevation < scheme.low {
        0.0
    } else if elevation < scheme.high {
        (scheme.low - elevation) / (scheme.low - scheme.high)
    } else {
        1.0
    }
}

/* Helper function to interpolate transition scheme.
   This is the same logic used in main.rs */
fn interpolate_transition_scheme(
    scheme: &TransitionScheme,
    progress: f64,
    result: &mut ColorSetting,
) {
    let alpha = progress.max(0.0).min(1.0);

    result.temperature = ((1.0 - alpha) * (scheme.night.temperature as f64)
        + alpha * (scheme.day.temperature as f64)) as i32;
    result.brightness = ((1.0 - alpha) * (scheme.night.brightness as f64)
        + alpha * (scheme.day.brightness as f64)) as f32;
    result.gamma[0] = ((1.0 - alpha) * (scheme.night.gamma[0] as f64)
        + alpha * (scheme.day.gamma[0] as f64)) as f32;
    result.gamma[1] = ((1.0 - alpha) * (scheme.night.gamma[1] as f64)
        + alpha * (scheme.day.gamma[1] as f64)) as f32;
    result.gamma[2] = ((1.0 - alpha) * (scheme.night.gamma[2] as f64)
        + alpha * (scheme.day.gamma[2] as f64)) as f32;
}

/* Helper function to check if color settings differ significantly */
fn color_setting_diff_is_major(first: &ColorSetting, second: &ColorSetting) -> bool {
    (first.temperature - second.temperature).abs() > 25
        || (first.brightness - second.brightness).abs() > 0.1
        || (first.gamma[0] - second.gamma[0]).abs() > 0.1
        || (first.gamma[1] - second.gamma[1]).abs() > 0.1
        || (first.gamma[2] - second.gamma[2]).abs() > 0.1
}

/* Helper function to interpolate between color settings */
fn interpolate_color_settings(
    first: &ColorSetting,
    second: &ColorSetting,
    alpha: f64,
    result: &mut ColorSetting,
) {
    let alpha = alpha.max(0.0).min(1.0);

    result.temperature = ((1.0 - alpha) * (first.temperature as f64)
        + alpha * (second.temperature as f64)) as i32;
    result.brightness = ((1.0 - alpha) * (first.brightness as f64)
        + alpha * (second.brightness as f64)) as f32;
    result.gamma[0] = ((1.0 - alpha) * (first.gamma[0] as f64)
        + alpha * (second.gamma[0] as f64)) as f32;
    result.gamma[1] = ((1.0 - alpha) * (first.gamma[1] as f64)
        + alpha * (second.gamma[1] as f64)) as f32;
    result.gamma[2] = ((1.0 - alpha) * (first.gamma[2] as f64)
        + alpha * (second.gamma[2] as f64)) as f32;
}

/* Helper function for cubic easing */
fn ease_fade(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

#[test]
fn test_transition_progress_at_night() {
    let scheme = TransitionScheme::default();
    // Elevation well below low threshold (-6.0)
    let elevation = -20.0;
    let progress = get_transition_progress_from_elevation(&scheme, elevation);
    assert_eq!(progress, 0.0, "Should return 0.0 for night period");
}

#[test]
fn test_transition_progress_at_day() {
    let scheme = TransitionScheme::default();
    // Elevation well above high threshold (3.0)
    let elevation = 10.0;
    let progress = get_transition_progress_from_elevation(&scheme, elevation);
    assert_eq!(progress, 1.0, "Should return 1.0 for day period");
}

#[test]
fn test_transition_progress_at_midpoint() {
    let scheme = TransitionScheme::default();
    // Elevation at exact midpoint between low (-6.0) and high (3.0)
    let elevation = -1.5;
    let progress = get_transition_progress_from_elevation(&scheme, elevation);
    assert!((progress - 0.5).abs() < 0.01, "Should return ~0.5 at midpoint");
}

#[test]
fn test_transition_progress_at_boundaries() {
    let scheme = TransitionScheme::default();

    // At low boundary
    let progress_low = get_transition_progress_from_elevation(&scheme, scheme.low);
    assert_eq!(progress_low, 0.0, "Should return 0.0 at low boundary");

    // At high boundary
    let progress_high = get_transition_progress_from_elevation(&scheme, scheme.high);
    assert_eq!(progress_high, 1.0, "Should return 1.0 at high boundary");
}

#[test]
fn test_transition_progress_increases_with_elevation() {
    let scheme = TransitionScheme::default();

    let prog1 = get_transition_progress_from_elevation(&scheme, -5.0);
    let prog2 = get_transition_progress_from_elevation(&scheme, -3.0);
    let prog3 = get_transition_progress_from_elevation(&scheme, -1.0);

    assert!(prog1 < prog2, "Progress should increase with elevation");
    assert!(prog2 < prog3, "Progress should increase with elevation");
}

#[test]
fn test_interpolate_scheme_at_night() {
    let scheme = TransitionScheme::default();
    let mut result = ColorSetting::default();

    interpolate_transition_scheme(&scheme, 0.0, &mut result);

    assert_eq!(result.temperature, scheme.night.temperature);
    assert_eq!(result.brightness, scheme.night.brightness);
    assert_eq!(result.gamma, scheme.night.gamma);
}

#[test]
fn test_interpolate_scheme_at_day() {
    let scheme = TransitionScheme::default();
    let mut result = ColorSetting::default();

    interpolate_transition_scheme(&scheme, 1.0, &mut result);

    assert_eq!(result.temperature, scheme.day.temperature);
    assert_eq!(result.brightness, scheme.day.brightness);
    assert_eq!(result.gamma, scheme.day.gamma);
}

#[test]
fn test_interpolate_scheme_at_midpoint() {
    let mut scheme = TransitionScheme::default();
    scheme.night.temperature = 3000;
    scheme.day.temperature = 6000;

    let mut result = ColorSetting::default();
    interpolate_transition_scheme(&scheme, 0.5, &mut result);

    let expected_temp = 4500;
    assert_eq!(result.temperature, expected_temp, "Should be midpoint temperature");
}

#[test]
fn test_interpolate_scheme_clamps_progress() {
    let scheme = TransitionScheme::default();
    let mut result1 = ColorSetting::default();
    let mut result2 = ColorSetting::default();

    // Test clamping below 0.0
    interpolate_transition_scheme(&scheme, -0.5, &mut result1);
    assert_eq!(result1.temperature, scheme.night.temperature);

    // Test clamping above 1.0
    interpolate_transition_scheme(&scheme, 1.5, &mut result2);
    assert_eq!(result2.temperature, scheme.day.temperature);
}

#[test]
fn test_color_diff_major_temperature() {
    let setting1 = ColorSetting {
        temperature: 6500,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let setting2 = ColorSetting {
        temperature: 6400,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };

    // Difference is 100K, which is > 25K threshold
    assert!(color_setting_diff_is_major(&setting1, &setting2));
}

#[test]
fn test_color_diff_minor_temperature() {
    let setting1 = ColorSetting {
        temperature: 6500,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let setting2 = ColorSetting {
        temperature: 6490,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };

    // Difference is 10K, which is < 25K threshold
    assert!(!color_setting_diff_is_major(&setting1, &setting2));
}

#[test]
fn test_color_diff_major_brightness() {
    let setting1 = ColorSetting {
        temperature: 6500,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let setting2 = ColorSetting {
        temperature: 6500,
        brightness: 0.8,
        gamma: [1.0, 1.0, 1.0],
    };

    // Difference is 0.2, which is > 0.1 threshold
    assert!(color_setting_diff_is_major(&setting1, &setting2));
}

#[test]
fn test_color_diff_major_gamma() {
    let setting1 = ColorSetting {
        temperature: 6500,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let setting2 = ColorSetting {
        temperature: 6500,
        brightness: 1.0,
        gamma: [0.8, 1.0, 1.0],
    };

    // Gamma R difference is 0.2, which is > 0.1 threshold
    assert!(color_setting_diff_is_major(&setting1, &setting2));
}

#[test]
fn test_interpolate_settings_at_start() {
    let first = ColorSetting {
        temperature: 3000,
        brightness: 0.8,
        gamma: [0.9, 0.9, 0.9],
    };
    let second = ColorSetting {
        temperature: 6000,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let mut result = ColorSetting::default();

    interpolate_color_settings(&first, &second, 0.0, &mut result);

    assert_eq!(result.temperature, first.temperature);
    assert_eq!(result.brightness, first.brightness);
    assert_eq!(result.gamma, first.gamma);
}

#[test]
fn test_interpolate_settings_at_end() {
    let first = ColorSetting {
        temperature: 3000,
        brightness: 0.8,
        gamma: [0.9, 0.9, 0.9],
    };
    let second = ColorSetting {
        temperature: 6000,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let mut result = ColorSetting::default();

    interpolate_color_settings(&first, &second, 1.0, &mut result);

    assert_eq!(result.temperature, second.temperature);
    assert_eq!(result.brightness, second.brightness);
    assert_eq!(result.gamma, second.gamma);
}

#[test]
fn test_interpolate_settings_at_midpoint() {
    let first = ColorSetting {
        temperature: 4000,
        brightness: 0.8,
        gamma: [0.8, 0.8, 0.8],
    };
    let second = ColorSetting {
        temperature: 6000,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let mut result = ColorSetting::default();

    interpolate_color_settings(&first, &second, 0.5, &mut result);

    assert_eq!(result.temperature, 5000);
    assert!((result.brightness - 0.9).abs() < 0.01);
    assert!((result.gamma[0] - 0.9).abs() < 0.01);
}

#[test]
fn test_interpolate_settings_clamps_alpha() {
    let first = ColorSetting {
        temperature: 3000,
        brightness: 0.8,
        gamma: [0.9, 0.9, 0.9],
    };
    let second = ColorSetting {
        temperature: 6000,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };

    let mut result_below = ColorSetting::default();
    interpolate_color_settings(&first, &second, -0.5, &mut result_below);
    assert_eq!(result_below.temperature, first.temperature);

    let mut result_above = ColorSetting::default();
    interpolate_color_settings(&first, &second, 1.5, &mut result_above);
    assert_eq!(result_above.temperature, second.temperature);
}

#[test]
fn test_ease_fade_at_boundaries() {
    assert_eq!(ease_fade(0.0), 0.0, "Should be 0.0 at start");
    assert_eq!(ease_fade(1.0), 1.0, "Should be 1.0 at end");
}

#[test]
fn test_ease_fade_at_midpoint() {
    let mid = ease_fade(0.5);
    assert_eq!(mid, 0.5, "Should be 0.5 at midpoint for cubic ease");
}

#[test]
fn test_ease_fade_is_smooth() {
    // Test that easing produces smooth acceleration/deceleration
    let t1 = ease_fade(0.25);
    let t2 = ease_fade(0.5);
    let t3 = ease_fade(0.75);

    // Should be monotonically increasing
    assert!(t1 < t2);
    assert!(t2 < t3);

    // Cubic easing should start slow, speed up, then slow down
    // So the first quarter should produce less than 0.25 progress
    assert!(t1 < 0.25);
    // And the last quarter should produce more than 0.25 progress
    assert!(t3 > 0.75);
}

#[test]
fn test_ease_fade_symmetric() {
    // Cubic ease should be symmetric around midpoint
    let early = ease_fade(0.3);
    let late = ease_fade(0.7);

    assert!((early + late - 1.0).abs() < 0.01, "Should be symmetric");
}

#[test]
fn test_fade_animation_sequence() {
    // Simulate a 40-step fade from day to night
    let start = ColorSetting {
        temperature: NEUTRAL_TEMP,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let target = ColorSetting {
        temperature: 3500,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };

    let fade_length = 40;
    let mut temps = Vec::new();

    for i in 0..=fade_length {
        let frac = i as f64 / fade_length as f64;
        let alpha = ease_fade(frac);
        let mut current = ColorSetting::default();
        interpolate_color_settings(&start, &target, alpha, &mut current);
        temps.push(current.temperature);
    }

    // First temp should be start temp
    assert_eq!(temps[0], start.temperature);

    // Last temp should be target temp
    assert_eq!(temps[fade_length as usize], target.temperature);

    // Temps should monotonically decrease
    for i in 1..temps.len() {
        assert!(temps[i] <= temps[i-1], "Temperature should decrease monotonically");
    }
}

#[test]
fn test_major_diff_triggers_fade() {
    // This tests the logic for when to start a fade
    let current = ColorSetting {
        temperature: NEUTRAL_TEMP,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let target = ColorSetting {
        temperature: 3500,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };

    // Temperature difference is 3000K, which should trigger fade
    assert!(color_setting_diff_is_major(&current, &target));
}

#[test]
fn test_minor_diff_no_fade() {
    let current = ColorSetting {
        temperature: 6500,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let target = ColorSetting {
        temperature: 6510,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };

    // Temperature difference is only 10K, should not trigger fade
    assert!(!color_setting_diff_is_major(&current, &target));
}
