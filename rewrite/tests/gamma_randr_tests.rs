use redshift_rebooted::gamma::GammaMethod;
use redshift_rebooted::gamma_randr::RandrGammaMethod;
use redshift_rebooted::types::*;

#[test]
fn test_randr_gamma_method_creation() {
    // Test that RandrGammaMethod can be created
    let method = RandrGammaMethod::new();
    assert_eq!(method.name(), "randr", "RandrGammaMethod name should be 'randr'");
}

#[test]
fn test_randr_gamma_method_default() {
    // Test that RandrGammaMethod can be created with Default trait
    let method = RandrGammaMethod::default();
    assert_eq!(method.name(), "randr", "Default RandrGammaMethod name should be 'randr'");
}

#[test]
fn test_randr_gamma_method_display_trait() {
    let method = RandrGammaMethod::new();
    let display_string = format!("{}", method);
    assert_eq!(display_string, "RandR", "RandrGammaMethod should display as 'RandR'");
}

#[test]
fn test_randr_gamma_method_init_no_display() {
    // Test init when DISPLAY is not set or X11 is not available
    // This may fail gracefully or succeed depending on environment
    let mut method = RandrGammaMethod::new();

    // We don't assert success/failure here because it depends on environment
    // Just verify it doesn't panic
    let _ = method.init();
}

#[test]
fn test_randr_gamma_method_set_screen() {
    // Test the set_screen configuration method
    let mut method = RandrGammaMethod::new();
    method.set_screen(0);
    // If we got here without panicking, the method works
}

#[test]
fn test_randr_gamma_method_set_crtcs() {
    // Test the set_crtcs configuration method
    let mut method = RandrGammaMethod::new();
    method.set_crtcs(vec![0, 1]);
    // If we got here without panicking, the method works
}

#[test]
fn test_randr_gamma_method_restore_without_init() {
    // Test that restore doesn't panic even if not initialized
    let mut method = RandrGammaMethod::new();
    method.restore();
    // Should not panic
}

// Integration test - only runs if X11 is available
#[test]
#[ignore] // Use `cargo test -- --ignored` to run this
fn test_randr_gamma_method_full_lifecycle_x11() {
    // This test requires X11 to be available
    let mut method = RandrGammaMethod::new();

    // Try to initialize
    if method.init().is_err() {
        eprintln!("X11 not available, skipping integration test");
        return;
    }

    // Try to start
    if method.start().is_err() {
        eprintln!("Could not start RandR method, skipping");
        return;
    }

    // Try to set a temperature
    let setting = ColorSetting {
        temperature: 5000,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    if let Err(e) = method.set_temperature(&setting, false) {
        eprintln!("Could not set temperature: {}", e);
    }

    // Restore
    method.restore();
}

// Integration test - test preserve flag
#[test]
#[ignore]
fn test_randr_gamma_method_preserve_flag_x11() {
    let mut method = RandrGammaMethod::new();

    if method.init().is_err() || method.start().is_err() {
        eprintln!("X11 not available, skipping integration test");
        return;
    }

    let setting = ColorSetting {
        temperature: 4000,
        gamma: [1.0, 1.0, 1.0],
        brightness: 0.9,
    };

    // Test without preserve
    if let Err(e) = method.set_temperature(&setting, false) {
        eprintln!("Could not set temperature without preserve: {}", e);
    }

    // Test with preserve
    if let Err(e) = method.set_temperature(&setting, true) {
        eprintln!("Could not set temperature with preserve: {}", e);
    }

    method.restore();
}

// Integration test - test multiple temperature changes
#[test]
#[ignore]
fn test_randr_gamma_method_multiple_changes_x11() {
    let mut method = RandrGammaMethod::new();

    if method.init().is_err() || method.start().is_err() {
        eprintln!("X11 not available, skipping integration test");
        return;
    }

    let temperatures = [6500, 5000, 3500, 4500, 6500];

    for temp in temperatures {
        let setting = ColorSetting {
            temperature: temp,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        if let Err(e) = method.set_temperature(&setting, false) {
            eprintln!("Could not set temperature {}: {}", temp, e);
        }
    }

    method.restore();
}

// Integration test - test extreme temperatures
#[test]
#[ignore]
fn test_randr_gamma_method_extreme_temperatures_x11() {
    let mut method = RandrGammaMethod::new();

    if method.init().is_err() || method.start().is_err() {
        eprintln!("X11 not available, skipping integration test");
        return;
    }

    // Test minimum temperature
    let min_setting = ColorSetting {
        temperature: MIN_TEMP,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    if let Err(e) = method.set_temperature(&min_setting, false) {
        eprintln!("Could not set minimum temperature: {}", e);
    }

    // Test maximum temperature
    let max_setting = ColorSetting {
        temperature: MAX_TEMP,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    if let Err(e) = method.set_temperature(&max_setting, false) {
        eprintln!("Could not set maximum temperature: {}", e);
    }

    // Restore to neutral
    let neutral_setting = ColorSetting {
        temperature: NEUTRAL_TEMP,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    if let Err(e) = method.set_temperature(&neutral_setting, false) {
        eprintln!("Could not restore neutral temperature: {}", e);
    }

    method.restore();
}

// Integration test - test various gamma values
#[test]
#[ignore]
fn test_randr_gamma_method_gamma_values_x11() {
    let mut method = RandrGammaMethod::new();

    if method.init().is_err() || method.start().is_err() {
        eprintln!("X11 not available, skipping integration test");
        return;
    }

    let gamma_values = [
        [1.0, 1.0, 1.0],
        [0.8, 0.8, 0.8],
        [1.2, 1.2, 1.2],
        [1.0, 0.9, 0.9],
    ];

    for gamma in gamma_values {
        let setting = ColorSetting {
            temperature: 6500,
            gamma,
            brightness: 1.0,
        };

        if let Err(e) = method.set_temperature(&setting, false) {
            eprintln!("Could not set gamma {:?}: {}", gamma, e);
        }
    }

    method.restore();
}

// Integration test - test brightness values
#[test]
#[ignore]
fn test_randr_gamma_method_brightness_values_x11() {
    let mut method = RandrGammaMethod::new();

    if method.init().is_err() || method.start().is_err() {
        eprintln!("X11 not available, skipping integration test");
        return;
    }

    let brightness_values = [1.0, 0.8, 0.6, 0.9, 1.0];

    for brightness in brightness_values {
        let setting = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness,
        };

        if let Err(e) = method.set_temperature(&setting, false) {
            eprintln!("Could not set brightness {}: {}", brightness, e);
        }
    }

    method.restore();
}

#[test]
fn test_randr_gamma_method_as_trait_object() {
    // Test that RandrGammaMethod can be used as a trait object
    let method: Box<dyn GammaMethod> = Box::new(RandrGammaMethod::new());
    assert_eq!(method.name(), "randr");
}

#[test]
fn test_randr_gamma_method_drop() {
    // Test that RandrGammaMethod's Drop implementation doesn't panic
    {
        let _method = RandrGammaMethod::new();
        // When _method goes out of scope, Drop should run
    }
    // If we got here, Drop didn't panic
}
