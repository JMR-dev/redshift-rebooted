use redshift_rebooted::gamma::*;
use redshift_rebooted::types::*;

#[test]
fn test_dummy_gamma_method_lifecycle() {
    // Test that DummyGammaMethod can be initialized and started
    let mut method = DummyGammaMethod::new();

    // Init should succeed
    assert!(method.init().is_ok(), "DummyGammaMethod init should succeed");

    // Start should succeed
    assert!(method.start().is_ok(), "DummyGammaMethod start should succeed");

    // Name should be "dummy"
    assert_eq!(method.name(), "dummy", "DummyGammaMethod name should be 'dummy'");
}

#[test]
fn test_dummy_gamma_method_set_temperature() {
    let mut method = DummyGammaMethod::new();
    method.init().unwrap();
    method.start().unwrap();

    // Create a color setting
    let setting = ColorSetting {
        temperature: 3500,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    // Set temperature should succeed (even though it's a no-op)
    assert!(
        method.set_temperature(&setting, false).is_ok(),
        "DummyGammaMethod set_temperature should succeed"
    );

    // Test with preserve flag
    assert!(
        method.set_temperature(&setting, true).is_ok(),
        "DummyGammaMethod set_temperature with preserve should succeed"
    );
}

#[test]
fn test_dummy_gamma_method_restore() {
    let mut method = DummyGammaMethod::new();
    method.init().unwrap();
    method.start().unwrap();

    // Restore should not panic (it's a no-op for dummy)
    method.restore();
}

#[test]
fn test_dummy_gamma_method_various_temperatures() {
    let mut method = DummyGammaMethod::new();
    method.init().unwrap();
    method.start().unwrap();

    // Test with various temperature values
    let temperatures = [1000, 2500, 3500, 5000, 6500, 10000, 25000];

    for temp in temperatures {
        let setting = ColorSetting {
            temperature: temp,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        assert!(
            method.set_temperature(&setting, false).is_ok(),
            "DummyGammaMethod should handle temperature {}K",
            temp
        );
    }
}

#[test]
fn test_dummy_gamma_method_various_gamma_values() {
    let mut method = DummyGammaMethod::new();
    method.init().unwrap();
    method.start().unwrap();

    // Test with various gamma values
    let gamma_values = [
        [0.5, 0.5, 0.5],
        [1.0, 1.0, 1.0],
        [1.5, 1.5, 1.5],
        [1.0, 0.8, 0.8],
    ];

    for gamma in gamma_values {
        let setting = ColorSetting {
            temperature: 6500,
            gamma,
            brightness: 1.0,
        };

        assert!(
            method.set_temperature(&setting, false).is_ok(),
            "DummyGammaMethod should handle gamma {:?}",
            gamma
        );
    }
}

#[test]
fn test_dummy_gamma_method_various_brightness_values() {
    let mut method = DummyGammaMethod::new();
    method.init().unwrap();
    method.start().unwrap();

    // Test with various brightness values
    let brightness_values = [0.1, 0.5, 0.75, 1.0];

    for brightness in brightness_values {
        let setting = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness,
        };

        assert!(
            method.set_temperature(&setting, false).is_ok(),
            "DummyGammaMethod should handle brightness {}",
            brightness
        );
    }
}

#[test]
fn test_dummy_gamma_method_multiple_calls() {
    let mut method = DummyGammaMethod::new();
    method.init().unwrap();
    method.start().unwrap();

    // Test calling set_temperature multiple times (simulating continual mode)
    let setting1 = ColorSetting {
        temperature: 6500,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    let setting2 = ColorSetting {
        temperature: 3500,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    assert!(method.set_temperature(&setting1, false).is_ok());
    assert!(method.set_temperature(&setting2, false).is_ok());
    assert!(method.set_temperature(&setting1, false).is_ok());
    assert!(method.set_temperature(&setting2, false).is_ok());
}

#[test]
fn test_gamma_method_trait_object() {
    // Test that GammaMethod can be used as a trait object
    let mut method: Box<dyn GammaMethod> = Box::new(DummyGammaMethod::new());

    assert!(method.init().is_ok());
    assert!(method.start().is_ok());

    let setting = ColorSetting {
        temperature: 4500,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    assert!(method.set_temperature(&setting, false).is_ok());
    method.restore();
}

#[test]
fn test_gamma_method_default_color_setting() {
    let mut method = DummyGammaMethod::new();
    method.init().unwrap();
    method.start().unwrap();

    // Test with default ColorSetting
    let setting = ColorSetting::default();

    assert_eq!(setting.temperature, NEUTRAL_TEMP);
    assert_eq!(setting.gamma, [1.0, 1.0, 1.0]);
    assert_eq!(setting.brightness, 1.0);

    assert!(
        method.set_temperature(&setting, false).is_ok(),
        "DummyGammaMethod should handle default ColorSetting"
    );
}

#[test]
fn test_dummy_gamma_method_display_trait() {
    let method = DummyGammaMethod::new();
    let display_string = format!("{}", method);
    assert_eq!(display_string, "Dummy", "DummyGammaMethod should display as 'Dummy'");
}
