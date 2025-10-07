/// Additional tests for gamma module to improve coverage

use redshift_rebooted::gamma::{DummyGammaMethod, GammaMethod};
use redshift_rebooted::types::ColorSetting;

#[test]
fn test_dummy_gamma_method_name() {
    let method = DummyGammaMethod::new();
    assert_eq!(method.name(), "dummy");
}

#[test]
fn test_dummy_gamma_method_print_help() {
    let method = DummyGammaMethod::new();

    // Should not panic
    method.print_help();
}

#[test]
fn test_dummy_gamma_method_with_preserve_flag() {
    let mut method = DummyGammaMethod::new();
    method.init().expect("Init should succeed");
    method.start().expect("Start should succeed");

    let setting = ColorSetting {
        temperature: 3500,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };

    // Test with preserve = true
    let result = method.set_temperature(&setting, true);
    assert!(result.is_ok(), "set_temperature with preserve=true should succeed");

    // Test with preserve = false
    let result = method.set_temperature(&setting, false);
    assert!(result.is_ok(), "set_temperature with preserve=false should succeed");
}

#[test]
fn test_dummy_gamma_method_extreme_temperatures() {
    let mut method = DummyGammaMethod::new();
    method.init().expect("Init should succeed");
    method.start().expect("Start should succeed");

    // Very cool temperature
    let cool_setting = ColorSetting {
        temperature: 1000,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let result = method.set_temperature(&cool_setting, false);
    assert!(result.is_ok(), "Very cool temperature should succeed");

    // Very warm temperature
    let warm_setting = ColorSetting {
        temperature: 25000,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };
    let result = method.set_temperature(&warm_setting, false);
    assert!(result.is_ok(), "Very warm temperature should succeed");
}

#[test]
fn test_dummy_gamma_method_various_brightness() {
    let mut method = DummyGammaMethod::new();
    method.init().expect("Init should succeed");
    method.start().expect("Start should succeed");

    let brightnesses = [0.1, 0.5, 0.8, 1.0];

    for brightness in brightnesses {
        let setting = ColorSetting {
            temperature: 6500,
            brightness,
            gamma: [1.0, 1.0, 1.0],
        };
        let result = method.set_temperature(&setting, false);
        assert!(result.is_ok(), "Brightness {} should succeed", brightness);
    }
}

#[test]
fn test_dummy_gamma_method_various_gamma_values() {
    let mut method = DummyGammaMethod::new();
    method.init().expect("Init should succeed");
    method.start().expect("Start should succeed");

    let gamma_values = [
        [0.5, 0.5, 0.5],
        [1.0, 1.0, 1.0],
        [1.5, 1.5, 1.5],
        [2.0, 2.0, 2.0],
        [1.0, 1.2, 0.8], // Asymmetric gamma
    ];

    for gamma in gamma_values {
        let setting = ColorSetting {
            temperature: 6500,
            brightness: 1.0,
            gamma,
        };
        let result = method.set_temperature(&setting, false);
        assert!(result.is_ok(), "Gamma {:?} should succeed", gamma);
    }
}

#[test]
fn test_dummy_gamma_method_multiple_restore_calls() {
    let mut method = DummyGammaMethod::new();
    method.init().expect("Init should succeed");
    method.start().expect("Start should succeed");

    // Multiple restore calls should not panic
    method.restore();
    method.restore();
    method.restore();
}

#[test]
fn test_dummy_gamma_method_restore_without_start() {
    let mut method = DummyGammaMethod::new();

    // Restore without start should not panic
    method.restore();
}

#[test]
fn test_dummy_gamma_method_as_trait_object() {
    let mut method: Box<dyn GammaMethod> = Box::new(DummyGammaMethod::new());

    assert!(method.init().is_ok());
    assert!(method.start().is_ok());
    assert_eq!(method.name(), "dummy");

    let setting = ColorSetting::default();
    assert!(method.set_temperature(&setting, false).is_ok());

    method.restore();
    method.print_help();
}

#[test]
fn test_dummy_gamma_method_sequence_of_different_settings() {
    let mut method = DummyGammaMethod::new();
    method.init().expect("Init should succeed");
    method.start().expect("Start should succeed");

    // Sequence of different settings simulating a day cycle
    let settings = [
        ColorSetting { temperature: 6500, brightness: 0.5, gamma: [1.0, 1.0, 1.0] },
        ColorSetting { temperature: 5000, brightness: 0.7, gamma: [1.0, 1.0, 1.0] },
        ColorSetting { temperature: 4000, brightness: 0.9, gamma: [1.0, 1.0, 1.0] },
        ColorSetting { temperature: 3500, brightness: 1.0, gamma: [1.0, 1.0, 1.0] },
        ColorSetting { temperature: 4000, brightness: 0.9, gamma: [1.0, 1.0, 1.0] },
        ColorSetting { temperature: 5000, brightness: 0.7, gamma: [1.0, 1.0, 1.0] },
        ColorSetting { temperature: 6500, brightness: 0.5, gamma: [1.0, 1.0, 1.0] },
    ];

    for setting in &settings {
        let result = method.set_temperature(setting, false);
        assert!(result.is_ok(), "Setting temperature to {} should succeed", setting.temperature);
    }
}

#[test]
fn test_dummy_gamma_method_init_multiple_times() {
    let mut method = DummyGammaMethod::new();

    // Init multiple times should succeed
    assert!(method.init().is_ok());
    assert!(method.init().is_ok());
    assert!(method.init().is_ok());
}

#[test]
fn test_dummy_gamma_method_start_multiple_times() {
    let mut method = DummyGammaMethod::new();
    method.init().expect("Init should succeed");

    // Start multiple times should succeed
    assert!(method.start().is_ok());
    assert!(method.start().is_ok());
}

#[test]
fn test_color_setting_default_for_gamma_method() {
    let mut method = DummyGammaMethod::new();
    method.init().expect("Init should succeed");
    method.start().expect("Start should succeed");

    // Test with default ColorSetting
    let setting = ColorSetting::default();
    let result = method.set_temperature(&setting, false);
    assert!(result.is_ok(), "Default ColorSetting should work");
}
