/// Configuration file support for Redshift
/// Stores user preferences and location data

use crate::types::Location;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub location: Option<SavedLocation>,
    pub last_geoclue_check: Option<u64>, // Unix timestamp
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedLocation {
    pub lat: f32,
    pub lon: f32,
    pub source: LocationSource,
    pub city_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LocationSource {
    Manual,
    Interactive,
    GeoClue2,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            location: None,
            last_geoclue_check: None,
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not determine config directory")?;
        let redshift_dir = config_dir.join("redshift");
        Ok(redshift_dir.join("config.toml"))
    }

    /// Load config from file
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Check if we should try GeoClue2 again (once per day)
    pub fn should_check_geoclue(&self) -> bool {
        if let Some(last_check) = self.last_geoclue_check {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Check once per day (86400 seconds)
            now - last_check > 86400
        } else {
            // Never checked before
            true
        }
    }

    /// Update the last GeoClue2 check timestamp
    pub fn update_geoclue_check(&mut self) {
        self.last_geoclue_check = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }

    /// Set location from a Location struct
    pub fn set_location(&mut self, location: Location, source: LocationSource, city_name: Option<String>) {
        self.location = Some(SavedLocation {
            lat: location.lat,
            lon: location.lon,
            source,
            city_name,
        });
    }

    /// Get location as a Location struct
    pub fn get_location(&self) -> Option<Location> {
        self.location.as_ref().map(|loc| Location {
            lat: loc.lat,
            lon: loc.lon,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.location.is_none());
        assert!(config.last_geoclue_check.is_none());
    }

    #[test]
    fn test_config_should_check_geoclue() {
        let mut config = Config::default();

        // Should check when never checked before
        assert!(config.should_check_geoclue());

        // Set recent check
        config.last_geoclue_check = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        // Should not check again immediately
        assert!(!config.should_check_geoclue());

        // Set old check (2 days ago)
        config.last_geoclue_check = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - (2 * 86400)
        );

        // Should check again
        assert!(config.should_check_geoclue());
    }

    #[test]
    fn test_config_location() {
        let mut config = Config::default();

        let location = Location {
            lat: 40.7128,
            lon: -74.0060,
        };

        config.set_location(
            location,
            LocationSource::Manual,
            Some("New York".to_string())
        );

        assert!(config.location.is_some());
        let saved_loc = config.get_location().unwrap();
        assert_eq!(saved_loc.lat, 40.7128);
        assert_eq!(saved_loc.lon, -74.0060);
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config.set_location(
            Location { lat: 51.5074, lon: -0.1278 },
            LocationSource::Interactive,
            Some("London".to_string())
        );

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("lat"));
        assert!(toml_str.contains("lon"));

        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        let loc = deserialized.get_location().unwrap();
        assert_eq!(loc.lat, 51.5074);
    }
}
