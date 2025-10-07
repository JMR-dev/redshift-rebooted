/// INI Configuration file support for Redshift
/// Parses redshift.conf files in INI format (matching the C version)

use crate::types::*;
use ini::Ini;
use std::path::PathBuf;

/// Configuration loaded from INI file
#[derive(Debug, Clone, Default)]
pub struct RedshiftConfig {
    /* Redshift section settings */
    pub temp_day: Option<i32>,
    pub temp_night: Option<i32>,
    pub fade: Option<bool>,
    pub brightness_day: Option<f32>,
    pub brightness_night: Option<f32>,
    pub gamma_day: Option<[f32; 3]>,
    pub gamma_night: Option<[f32; 3]>,
    pub elevation_high: Option<f64>,
    pub elevation_low: Option<f64>,
    pub dawn_time: Option<TimeRange>,
    pub dusk_time: Option<TimeRange>,
    pub location_provider: Option<String>,
    pub adjustment_method: Option<String>,

    /* Manual location section */
    pub manual_lat: Option<f32>,
    pub manual_lon: Option<f32>,

    /* Gamma method settings */
    pub randr_screen: Option<i32>,
    pub randr_crtc: Option<i32>,
}

impl RedshiftConfig {
    /// Find and load the INI config file from standard locations
    pub fn load() -> Result<Self, String> {
        if let Some(path) = Self::find_config_file() {
            Self::load_from_file(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// Find the config file in standard XDG locations
    pub fn find_config_file() -> Option<PathBuf> {
        let paths = Self::get_config_search_paths();

        for path in paths {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Get list of paths to search for config file (in priority order)
    pub fn get_config_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        /* Priority 1: XDG_CONFIG_HOME/redshift/redshift.conf */
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg_config.is_empty() {
                paths.push(PathBuf::from(&xdg_config).join("redshift").join("redshift.conf"));
                /* Fallback: XDG_CONFIG_HOME/redshift.conf */
                paths.push(PathBuf::from(&xdg_config).join("redshift.conf"));
            }
        }

        /* Priority 2: HOME/.config/redshift/redshift.conf */
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".config").join("redshift").join("redshift.conf"));
            /* Fallback: HOME/.config/redshift.conf */
            paths.push(home_dir.join(".config").join("redshift.conf"));
        }

        /* Priority 3: System-wide configs */
        paths.push(PathBuf::from("/etc/redshift/redshift.conf"));
        paths.push(PathBuf::from("/etc/redshift.conf"));

        paths
    }

    /// Load config from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let ini = Ini::load_from_file(path)
            .map_err(|e| format!("Failed to load INI file: {}", e))?;

        let mut config = Self::default();

        /* Parse [redshift] section */
        if let Some(section) = ini.section(Some("redshift")) {
            if let Some(val) = section.get("temp-day") {
                config.temp_day = val.parse().ok();
            }
            if let Some(val) = section.get("temp-night") {
                config.temp_night = val.parse().ok();
            }
            if let Some(val) = section.get("fade") {
                config.fade = match val {
                    "0" => Some(false),
                    "1" => Some(true),
                    _ => val.parse().ok(),
                };
            }
            if let Some(val) = section.get("transition") {
                config.fade = match val {
                    "0" => Some(false),
                    "1" => Some(true),
                    _ => val.parse().ok(),
                };
            }

            /* Brightness settings */
            if let Some(val) = section.get("brightness") {
                if let Ok((day, night)) = parse_brightness_string(val) {
                    config.brightness_day = Some(day);
                    config.brightness_night = Some(night);
                }
            }
            if let Some(val) = section.get("brightness-day") {
                config.brightness_day = val.parse().ok();
            }
            if let Some(val) = section.get("brightness-night") {
                config.brightness_night = val.parse().ok();
            }

            /* Gamma settings */
            if let Some(val) = section.get("gamma") {
                if let Ok(gamma) = parse_gamma_string(val) {
                    config.gamma_day = Some(gamma);
                    config.gamma_night = Some(gamma);
                }
            }
            if let Some(val) = section.get("gamma-day") {
                if let Ok(gamma) = parse_gamma_string(val) {
                    config.gamma_day = Some(gamma);
                }
            }
            if let Some(val) = section.get("gamma-night") {
                if let Ok(gamma) = parse_gamma_string(val) {
                    config.gamma_night = Some(gamma);
                }
            }

            /* Elevation settings */
            if let Some(val) = section.get("elevation-high") {
                config.elevation_high = val.parse().ok();
            }
            if let Some(val) = section.get("elevation-low") {
                config.elevation_low = val.parse().ok();
            }

            /* Time-based transition settings */
            if let Some(val) = section.get("dawn-time") {
                config.dawn_time = parse_time_range(val).ok();
            }
            if let Some(val) = section.get("dusk-time") {
                config.dusk_time = parse_time_range(val).ok();
            }

            /* Provider/method settings */
            if let Some(val) = section.get("location-provider") {
                config.location_provider = Some(val.to_string());
            }
            if let Some(val) = section.get("adjustment-method") {
                config.adjustment_method = Some(val.to_string());
            }
        }

        /* Parse [manual] section for location */
        if let Some(section) = ini.section(Some("manual")) {
            if let Some(val) = section.get("lat") {
                config.manual_lat = val.parse().ok();
            }
            if let Some(val) = section.get("lon") {
                config.manual_lon = val.parse().ok();
            }
        }

        /* Parse [randr] section for gamma method settings */
        if let Some(section) = ini.section(Some("randr")) {
            if let Some(val) = section.get("screen") {
                config.randr_screen = val.parse().ok();
            }
            if let Some(val) = section.get("crtc") {
                config.randr_crtc = val.parse().ok();
            }
        }

        Ok(config)
    }

    /// Get manual location if specified
    pub fn get_manual_location(&self) -> Option<Location> {
        if let (Some(lat), Some(lon)) = (self.manual_lat, self.manual_lon) {
            Some(Location { lat, lon })
        } else {
            None
        }
    }
}

/// Parse brightness string: "0.9" or "0.7:0.4" (day:night)
pub fn parse_brightness_string(s: &str) -> Result<(f32, f32), String> {
    let parts: Vec<&str> = s.split(':').collect();

    if parts.len() == 1 {
        /* Same value for day and night */
        let val: f32 = parts[0].parse()
            .map_err(|_| format!("Invalid brightness value: {}", parts[0]))?;
        Ok((val, val))
    } else if parts.len() == 2 {
        /* Separate values for day and night */
        let day: f32 = parts[0].parse()
            .map_err(|_| format!("Invalid day brightness: {}", parts[0]))?;
        let night: f32 = parts[1].parse()
            .map_err(|_| format!("Invalid night brightness: {}", parts[1]))?;
        Ok((day, night))
    } else {
        Err("Brightness must be single value or day:night".to_string())
    }
}

/// Parse gamma string: "0.8" or "0.8:0.7:0.8" (R:G:B)
pub fn parse_gamma_string(s: &str) -> Result<[f32; 3], String> {
    let parts: Vec<&str> = s.split(':').collect();

    if parts.len() == 1 {
        /* Use same value for all channels */
        let val: f32 = parts[0].parse()
            .map_err(|_| format!("Invalid gamma value: {}", parts[0]))?;
        Ok([val, val, val])
    } else if parts.len() == 3 {
        /* Separate values for R, G, B */
        let r: f32 = parts[0].parse()
            .map_err(|_| format!("Invalid red gamma: {}", parts[0]))?;
        let g: f32 = parts[1].parse()
            .map_err(|_| format!("Invalid green gamma: {}", parts[1]))?;
        let b: f32 = parts[2].parse()
            .map_err(|_| format!("Invalid blue gamma: {}", parts[2]))?;
        Ok([r, g, b])
    } else {
        Err("Gamma must be single value or R:G:B".to_string())
    }
}

/// Parse time range string: "6:00" or "6:00-7:45"
fn parse_time_range(s: &str) -> Result<TimeRange, String> {
    let parts: Vec<&str> = s.split('-').collect();

    let start_time = parse_time(parts[0])?;
    let end_time = if parts.len() == 2 {
        parse_time(parts[1])?
    } else if parts.len() == 1 {
        start_time
    } else {
        return Err("Time range must be HH:MM or HH:MM-HH:MM".to_string());
    };

    Ok(TimeRange {
        start: start_time,
        end: end_time,
    })
}

/// Parse time string "HH:MM" to seconds since midnight
fn parse_time(s: &str) -> Result<i32, String> {
    let parts: Vec<&str> = s.split(':').collect();

    if parts.len() != 2 {
        return Err(format!("Time must be in HH:MM format: {}", s));
    }

    let hours: i32 = parts[0].parse()
        .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
    let minutes: i32 = parts[1].parse()
        .map_err(|_| format!("Invalid minute: {}", parts[1]))?;

    if hours < 0 || hours >= 24 {
        return Err(format!("Hours must be 0-23: {}", hours));
    }
    if minutes < 0 || minutes >= 60 {
        return Err(format!("Minutes must be 0-59: {}", minutes));
    }

    Ok(hours * 3600 + minutes * 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_brightness_single() {
        let (day, night) = parse_brightness_string("0.9").unwrap();
        assert_eq!(day, 0.9);
        assert_eq!(night, 0.9);
    }

    #[test]
    fn test_parse_brightness_separate() {
        let (day, night) = parse_brightness_string("0.7:0.4").unwrap();
        assert_eq!(day, 0.7);
        assert_eq!(night, 0.4);
    }

    #[test]
    fn test_parse_gamma_single() {
        let gamma = parse_gamma_string("0.8").unwrap();
        assert_eq!(gamma, [0.8, 0.8, 0.8]);
    }

    #[test]
    fn test_parse_gamma_rgb() {
        let gamma = parse_gamma_string("0.8:0.7:0.9").unwrap();
        assert_eq!(gamma, [0.8, 0.7, 0.9]);
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("6:00").unwrap(), 6 * 3600);
        assert_eq!(parse_time("18:30").unwrap(), 18 * 3600 + 30 * 60);
    }

    #[test]
    fn test_parse_time_range() {
        let range = parse_time_range("6:00-7:45").unwrap();
        assert_eq!(range.start, 6 * 3600);
        assert_eq!(range.end, 7 * 3600 + 45 * 60);
    }

    #[test]
    fn test_parse_time_range_single() {
        let range = parse_time_range("6:00").unwrap();
        assert_eq!(range.start, 6 * 3600);
        assert_eq!(range.end, 6 * 3600);
    }
}
