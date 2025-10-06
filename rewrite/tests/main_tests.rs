/// Tests for main.rs location parsing and determination logic

use redshift_rebooted::types::*;

// Helper to parse location string (mimics main.rs parse_location)
fn parse_location(loc_str: &str) -> Result<Location, String> {
    let parts: Vec<&str> = loc_str.split(':').collect();
    if parts.len() != 2 {
        return Err("Location must be in format LAT:LON".to_string());
    }

    let lat: f32 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid latitude: {}", parts[0]))?;
    let lon: f32 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid longitude: {}", parts[1]))?;

    if lat < MIN_LAT || lat > MAX_LAT {
        return Err(format!(
            "Latitude must be between {} and {}",
            MIN_LAT, MAX_LAT
        ));
    }
    if lon < MIN_LON || lon > MAX_LON {
        return Err(format!(
            "Longitude must be between {} and {}",
            MIN_LON, MAX_LON
        ));
    }

    Ok(Location { lat, lon })
}

#[cfg(test)]
mod parse_location_tests {
    use super::*;

    #[test]
    fn test_parse_valid_location_positive() {
        let result = parse_location("40.7:-74.0");
        assert!(result.is_ok());
        let loc = result.unwrap();
        assert_eq!(loc.lat, 40.7);
        assert_eq!(loc.lon, -74.0);
    }

    #[test]
    fn test_parse_valid_location_negative() {
        let result = parse_location("-33.9:151.2");
        assert!(result.is_ok());
        let loc = result.unwrap();
        assert_eq!(loc.lat, -33.9);
        assert_eq!(loc.lon, 151.2);
    }

    #[test]
    fn test_parse_valid_location_zero() {
        let result = parse_location("0:0");
        assert!(result.is_ok());
        let loc = result.unwrap();
        assert_eq!(loc.lat, 0.0);
        assert_eq!(loc.lon, 0.0);
    }

    #[test]
    fn test_parse_location_boundary_values() {
        // Test max latitude
        let result = parse_location("90:0");
        assert!(result.is_ok());

        // Test min latitude
        let result = parse_location("-90:0");
        assert!(result.is_ok());

        // Test max longitude
        let result = parse_location("0:180");
        assert!(result.is_ok());

        // Test min longitude
        let result = parse_location("0:-180");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_location_invalid_format_no_colon() {
        let result = parse_location("40.7");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("LAT:LON"));
    }

    #[test]
    fn test_parse_location_invalid_format_too_many_colons() {
        let result = parse_location("40:74:0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_location_invalid_latitude_not_a_number() {
        let result = parse_location("abc:74.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid latitude"));
    }

    #[test]
    fn test_parse_location_invalid_longitude_not_a_number() {
        let result = parse_location("40.7:xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid longitude"));
    }

    #[test]
    fn test_parse_location_latitude_too_high() {
        let result = parse_location("91:0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Latitude must be between"));
    }

    #[test]
    fn test_parse_location_latitude_too_low() {
        let result = parse_location("-91:0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Latitude must be between"));
    }

    #[test]
    fn test_parse_location_longitude_too_high() {
        let result = parse_location("0:181");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Longitude must be between"));
    }

    #[test]
    fn test_parse_location_longitude_too_low() {
        let result = parse_location("0:-181");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Longitude must be between"));
    }

    #[test]
    fn test_parse_location_with_decimal_precision() {
        let result = parse_location("40.7128:-74.0060");
        assert!(result.is_ok());
        let loc = result.unwrap();
        assert!((loc.lat - 40.7128).abs() < 0.0001);
        assert!((loc.lon - (-74.0060)).abs() < 0.0001);
    }

    #[test]
    fn test_parse_location_empty_string() {
        let result = parse_location("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_location_only_colon() {
        let result = parse_location(":");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod interpolation_tests {
    use super::*;

    fn interpolate_color_setting(
        elevation: f64,
        low: f64,
        high: f64,
        night: &ColorSetting,
        day: &ColorSetting,
    ) -> ColorSetting {
        let alpha = ((elevation - low) / (high - low)) as f32;
        let alpha = alpha.max(0.0).min(1.0);

        ColorSetting {
            temperature: ((1.0 - alpha) * (night.temperature as f32) + alpha * (day.temperature as f32))
                as i32,
            gamma: [
                (1.0 - alpha) * night.gamma[0] + alpha * day.gamma[0],
                (1.0 - alpha) * night.gamma[1] + alpha * day.gamma[1],
                (1.0 - alpha) * night.gamma[2] + alpha * day.gamma[2],
            ],
            brightness: (1.0 - alpha) * night.brightness + alpha * day.brightness,
        }
    }

    #[test]
    fn test_interpolate_at_low_elevation() {
        let night = ColorSetting {
            temperature: 3500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let day = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        let result = interpolate_color_setting(-6.0, -6.0, 3.0, &night, &day);
        assert_eq!(result.temperature, 3500);
    }

    #[test]
    fn test_interpolate_at_high_elevation() {
        let night = ColorSetting {
            temperature: 3500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let day = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        let result = interpolate_color_setting(3.0, -6.0, 3.0, &night, &day);
        assert_eq!(result.temperature, 6500);
    }

    #[test]
    fn test_interpolate_midpoint() {
        let night = ColorSetting {
            temperature: 3500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let day = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        let result = interpolate_color_setting(-1.5, -6.0, 3.0, &night, &day);
        // Midpoint between 3500 and 6500 is 5000
        assert_eq!(result.temperature, 5000);
    }

    #[test]
    fn test_interpolate_brightness() {
        let night = ColorSetting {
            temperature: 3500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 0.5,
        };
        let day = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        let result = interpolate_color_setting(-1.5, -6.0, 3.0, &night, &day);
        assert!((result.brightness - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_gamma() {
        let night = ColorSetting {
            temperature: 3500,
            gamma: [0.8, 0.8, 0.8],
            brightness: 1.0,
        };
        let day = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        let result = interpolate_color_setting(-1.5, -6.0, 3.0, &night, &day);
        assert!((result.gamma[0] - 0.9).abs() < 0.01);
        assert!((result.gamma[1] - 0.9).abs() < 0.01);
        assert!((result.gamma[2] - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_below_range_clamps() {
        let night = ColorSetting {
            temperature: 3500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let day = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        // Elevation below low should clamp to night
        let result = interpolate_color_setting(-10.0, -6.0, 3.0, &night, &day);
        assert_eq!(result.temperature, 3500);
    }

    #[test]
    fn test_interpolate_above_range_clamps() {
        let night = ColorSetting {
            temperature: 3500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let day = ColorSetting {
            temperature: 6500,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        // Elevation above high should clamp to day
        let result = interpolate_color_setting(10.0, -6.0, 3.0, &night, &day);
        assert_eq!(result.temperature, 6500);
    }
}

#[cfg(test)]
mod color_setting_tests {
    use super::*;

    fn color_setting_diff_is_major(first: &ColorSetting, second: &ColorSetting) -> bool {
        (first.temperature - second.temperature).abs() > 25
            || (first.brightness - second.brightness).abs() > 0.1
            || (first.gamma[0] - second.gamma[0]).abs() > 0.1
            || (first.gamma[1] - second.gamma[1]).abs() > 0.1
            || (first.gamma[2] - second.gamma[2]).abs() > 0.1
    }

    #[test]
    fn test_identical_settings_not_major() {
        let setting = ColorSetting {
            temperature: 5000,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        assert!(!color_setting_diff_is_major(&setting, &setting));
    }

    #[test]
    fn test_small_temperature_diff_not_major() {
        let first = ColorSetting {
            temperature: 5000,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let second = ColorSetting {
            temperature: 5020,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        assert!(!color_setting_diff_is_major(&first, &second));
    }

    #[test]
    fn test_large_temperature_diff_is_major() {
        let first = ColorSetting {
            temperature: 5000,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let second = ColorSetting {
            temperature: 5100,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        assert!(color_setting_diff_is_major(&first, &second));
    }

    #[test]
    fn test_brightness_diff_is_major() {
        let first = ColorSetting {
            temperature: 5000,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let second = ColorSetting {
            temperature: 5000,
            gamma: [1.0, 1.0, 1.0],
            brightness: 0.8,
        };

        assert!(color_setting_diff_is_major(&first, &second));
    }

    #[test]
    fn test_gamma_diff_is_major() {
        let first = ColorSetting {
            temperature: 5000,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let second = ColorSetting {
            temperature: 5000,
            gamma: [0.85, 1.0, 1.0],
            brightness: 1.0,
        };

        assert!(color_setting_diff_is_major(&first, &second));
    }

    #[test]
    fn test_boundary_temperature_25k_not_major() {
        let first = ColorSetting {
            temperature: 5000,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let second = ColorSetting {
            temperature: 5025,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        assert!(!color_setting_diff_is_major(&first, &second));
    }

    #[test]
    fn test_boundary_temperature_26k_is_major() {
        let first = ColorSetting {
            temperature: 5000,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };
        let second = ColorSetting {
            temperature: 5026,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        };

        assert!(color_setting_diff_is_major(&first, &second));
    }
}

#[cfg(test)]
mod ease_fade_tests {
    fn ease_fade(t: f64) -> f64 {
        t * t * (3.0 - 2.0 * t)
    }

    #[test]
    fn test_ease_fade_at_zero() {
        assert_eq!(ease_fade(0.0), 0.0);
    }

    #[test]
    fn test_ease_fade_at_one() {
        assert_eq!(ease_fade(1.0), 1.0);
    }

    #[test]
    fn test_ease_fade_at_half() {
        let result = ease_fade(0.5);
        assert!((result - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_ease_fade_smooth_curve() {
        // Ease function should be smooth (no sudden jumps)
        let t1 = ease_fade(0.3);
        let t2 = ease_fade(0.31);
        assert!((t2 - t1).abs() < 0.1); // Should change gradually
    }

    #[test]
    fn test_ease_fade_monotonic_increasing() {
        // Function should always increase
        for i in 0..100 {
            let t1 = i as f64 / 100.0;
            let t2 = (i + 1) as f64 / 100.0;
            assert!(ease_fade(t2) >= ease_fade(t1));
        }
    }
}
