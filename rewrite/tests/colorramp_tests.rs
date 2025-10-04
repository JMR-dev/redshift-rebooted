use redshift_rebooted::colorramp::*;
use redshift_rebooted::types::*;

const EPSILON: f32 = 0.01;

#[test]
fn test_get_white_point_neutral() {
    // At 6500K (neutral), white point should be close to [1.0, 1.0, 1.0]
    let white_point = get_white_point(6500);

    assert!(
        (white_point[0] - 1.0).abs() < EPSILON,
        "Red channel at 6500K should be ~1.0, got {}",
        white_point[0]
    );
    assert!(
        (white_point[1] - 1.0).abs() < EPSILON,
        "Green channel at 6500K should be ~1.0, got {}",
        white_point[1]
    );
    assert!(
        (white_point[2] - 1.0).abs() < EPSILON,
        "Blue channel at 6500K should be ~1.0, got {}",
        white_point[2]
    );
}

#[test]
fn test_get_white_point_warm() {
    // At 3500K (warm/night), blue should be reduced, red should be high
    let white_point = get_white_point(3500);

    assert!(
        white_point[0] > 0.9,
        "Red channel at 3500K should be high, got {}",
        white_point[0]
    );
    // At 3500K, blue is around 0.55 (moderately reduced from 1.0)
    assert!(
        white_point[2] > 0.5 && white_point[2] < 0.6,
        "Blue channel at 3500K should be ~0.55, got {}",
        white_point[2]
    );
}

#[test]
fn test_get_white_point_cool() {
    // At 10000K (cool/blue), blue should be high, red should be lower
    let white_point = get_white_point(10000);

    assert!(
        white_point[0] < 0.9,
        "Red channel at 10000K should be reduced, got {}",
        white_point[0]
    );
    assert!(
        (white_point[2] - 1.0).abs() < EPSILON,
        "Blue channel at 10000K should be ~1.0, got {}",
        white_point[2]
    );
}

#[test]
fn test_get_white_point_interpolation() {
    // Test that interpolation works correctly
    // 6450K should be halfway between 6400K and 6500K entries
    let wp_6400 = get_white_point(6400);
    let wp_6450 = get_white_point(6450);
    let wp_6500 = get_white_point(6500);

    // 6450 should be approximately midway between 6400 and 6500
    for i in 0..3 {
        let expected_mid = (wp_6400[i] + wp_6500[i]) / 2.0;
        assert!(
            (wp_6450[i] - expected_mid).abs() < 0.05,
            "Channel {} at 6450K should interpolate between 6400K and 6500K",
            i
        );
    }
}

#[test]
fn test_get_white_point_boundaries() {
    // Test at boundaries of the table (1000K and 25000K)
    let wp_low = get_white_point(1000);
    let wp_high = get_white_point(25000);

    // Low temp: high red, very low blue
    assert!(wp_low[0] > 0.9, "1000K should have high red");
    assert!(wp_low[2] < 0.1, "1000K should have very low blue");

    // High temp: lower red, high blue
    assert!(wp_high[0] < 0.7, "25000K should have reduced red");
    assert!(
        (wp_high[2] - 1.0).abs() < EPSILON,
        "25000K should have blue ~1.0"
    );
}

#[test]
fn test_colorramp_fill_neutral_no_adjustment() {
    // With neutral temperature and default settings, ramp should be unchanged
    let size = 256;
    let mut gamma_r = vec![0u16; size];
    let mut gamma_g = vec![0u16; size];
    let mut gamma_b = vec![0u16; size];

    // Initialize with linear ramp
    for i in 0..size {
        let val = ((i * 65535) / (size - 1)) as u16;
        gamma_r[i] = val;
        gamma_g[i] = val;
        gamma_b[i] = val;
    }

    let setting = ColorSetting::default(); // 6500K, gamma 1.0, brightness 1.0

    let original_r = gamma_r.clone();
    let original_g = gamma_g.clone();
    let original_b = gamma_b.clone();

    colorramp_fill(&mut gamma_r, &mut gamma_g, &mut gamma_b, &setting);

    // With neutral settings, values should be very close to original
    for i in 0..size {
        let diff_r = (gamma_r[i] as i32 - original_r[i] as i32).abs();
        let diff_g = (gamma_g[i] as i32 - original_g[i] as i32).abs();
        let diff_b = (gamma_b[i] as i32 - original_b[i] as i32).abs();

        assert!(
            diff_r < 500,
            "Red channel should be nearly unchanged at neutral settings"
        );
        assert!(
            diff_g < 500,
            "Green channel should be nearly unchanged at neutral settings"
        );
        assert!(
            diff_b < 500,
            "Blue channel should be nearly unchanged at neutral settings"
        );
    }
}

#[test]
fn test_colorramp_fill_warm_reduces_blue() {
    // Warm temperature (3500K) should significantly reduce blue channel
    let size = 256;
    let mut gamma_r = vec![0u16; size];
    let mut gamma_g = vec![0u16; size];
    let mut gamma_b = vec![0u16; size];

    // Initialize with linear ramp
    for i in 0..size {
        let val = ((i * 65535) / (size - 1)) as u16;
        gamma_r[i] = val;
        gamma_g[i] = val;
        gamma_b[i] = val;
    }

    let original_b = gamma_b.clone();

    let setting = ColorSetting {
        temperature: 3500,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    colorramp_fill(&mut gamma_r, &mut gamma_g, &mut gamma_b, &setting);

    // Blue channel should be reduced (to about 35% at 3500K)
    for i in size / 2..size {
        // Test upper half where values are more significant
        // At 3500K, blue multiplier is ~0.35, so values should be less than original
        assert!(
            gamma_b[i] < original_b[i],
            "Blue channel should be reduced at 3500K at index {}",
            i
        );
    }
}

#[test]
fn test_colorramp_fill_brightness() {
    // Test brightness adjustment
    let size = 256;
    let mut gamma_r = vec![0u16; size];
    let mut gamma_g = vec![0u16; size];
    let mut gamma_b = vec![0u16; size];

    for i in 0..size {
        let val = ((i * 65535) / (size - 1)) as u16;
        gamma_r[i] = val;
        gamma_g[i] = val;
        gamma_b[i] = val;
    }

    let original_r = gamma_r.clone();

    let setting = ColorSetting {
        temperature: 6500,
        gamma: [1.0, 1.0, 1.0],
        brightness: 0.5, // Half brightness
    };

    colorramp_fill(&mut gamma_r, &mut gamma_g, &mut gamma_b, &setting);

    // With half brightness, values should be roughly halved
    for i in size / 2..size {
        let expected = original_r[i] / 2;
        let actual = gamma_r[i];
        let diff = (actual as i32 - expected as i32).abs();

        assert!(
            diff < 5000,
            "Brightness 0.5 should roughly halve values at index {}",
            i
        );
    }
}

#[test]
fn test_colorramp_fill_gamma() {
    // Test gamma adjustment
    let size = 256;
    let mut gamma_r = vec![0u16; size];
    let mut gamma_g = vec![0u16; size];
    let mut gamma_b = vec![0u16; size];

    for i in 0..size {
        let val = ((i * 65535) / (size - 1)) as u16;
        gamma_r[i] = val;
        gamma_g[i] = val;
        gamma_b[i] = val;
    }

    let original_r = gamma_r.clone();

    let setting = ColorSetting {
        temperature: 6500,
        gamma: [2.0, 1.0, 1.0], // Higher gamma for red
        brightness: 1.0,
    };

    colorramp_fill(&mut gamma_r, &mut gamma_g, &mut gamma_b, &setting);

    // Gamma 2.0 means output = input^(1/2.0) = sqrt(input)
    // This makes midtones darker, but max value (1.0) stays at 1.0
    // Check that middle values are affected
    let mid_idx = size / 2;
    assert!(
        gamma_r[mid_idx] != original_r[mid_idx],
        "Gamma 2.0 should affect middle values"
    );
}

#[test]
fn test_colorramp_fill_float_neutral() {
    // Test float version with neutral settings
    let size = 256;
    let mut gamma_r = vec![0.0f32; size];
    let mut gamma_g = vec![0.0f32; size];
    let mut gamma_b = vec![0.0f32; size];

    // Initialize with linear ramp from 0.0 to 1.0
    for i in 0..size {
        let val = (i as f32) / ((size - 1) as f32);
        gamma_r[i] = val;
        gamma_g[i] = val;
        gamma_b[i] = val;
    }

    let setting = ColorSetting::default();
    let original_r = gamma_r.clone();

    colorramp_fill_float(&mut gamma_r, &mut gamma_g, &mut gamma_b, &setting);

    // With neutral settings, values should be very close to original
    for i in 0..size {
        let diff = (gamma_r[i] - original_r[i]).abs();
        assert!(
            diff < 0.05,
            "Float ramp should be nearly unchanged at neutral settings at index {}",
            i
        );
    }
}

#[test]
fn test_colorramp_fill_float_warm() {
    // Test float version with warm temperature
    let size = 256;
    let mut gamma_r = vec![0.0f32; size];
    let mut gamma_g = vec![0.0f32; size];
    let mut gamma_b = vec![0.0f32; size];

    for i in 0..size {
        let val = (i as f32) / ((size - 1) as f32);
        gamma_r[i] = val;
        gamma_g[i] = val;
        gamma_b[i] = val;
    }

    let original_b = gamma_b.clone();

    let setting = ColorSetting {
        temperature: 3500,
        gamma: [1.0, 1.0, 1.0],
        brightness: 1.0,
    };

    colorramp_fill_float(&mut gamma_r, &mut gamma_g, &mut gamma_b, &setting);

    // Blue should be reduced (multiplied by ~0.35 at 3500K)
    for i in size / 2..size {
        assert!(
            gamma_b[i] < original_b[i],
            "Float: Blue should be reduced at 3500K at index {}",
            i
        );
    }
}

#[test]
fn test_temperature_progression() {
    // Test that color temperature changes smoothly across range
    let temps = [2000, 3500, 5000, 6500, 8000, 10000];
    let mut blue_values = Vec::new();

    for &temp in &temps {
        let wp = get_white_point(temp);
        blue_values.push(wp[2]);
    }

    // Blue channel should generally increase with temperature
    for i in 1..blue_values.len() {
        assert!(
            blue_values[i] >= blue_values[i - 1] - 0.01,
            "Blue channel should increase with temperature (or stay same)"
        );
    }
}

#[test]
fn test_color_setting_cloning() {
    let setting = ColorSetting {
        temperature: 5000,
        gamma: [0.9, 1.0, 1.1],
        brightness: 0.8,
    };

    let cloned = setting;

    assert_eq!(setting.temperature, cloned.temperature);
    assert_eq!(setting.gamma, cloned.gamma);
    assert_eq!(setting.brightness, cloned.brightness);
}
