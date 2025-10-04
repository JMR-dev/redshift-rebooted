/// Location providers
/// Ported from legacy/src/location-*.c

use crate::types::Location;

/// Trait for location providers
pub trait LocationProvider {
    /// Initialize the provider
    fn init(&mut self) -> Result<(), String>;

    /// Start the provider
    fn start(&mut self) -> Result<(), String>;

    /// Get the current location
    fn get_location(&mut self) -> Result<Location, String>;

    /// Get the provider name
    fn name(&self) -> &str;

    /// Print help information
    fn print_help(&self);

    /// Set an option (key-value pair)
    fn set_option(&mut self, key: &str, value: &str) -> Result<(), String>;
}

/// Manual location provider
/// Ported from legacy/src/location-manual.c
pub struct ManualLocationProvider {
    location: Option<Location>,
}

impl ManualLocationProvider {
    pub fn new() -> Self {
        Self { location: None }
    }

    pub fn with_location(lat: f32, lon: f32) -> Self {
        Self {
            location: Some(Location { lat, lon }),
        }
    }
}

impl Default for ManualLocationProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationProvider for ManualLocationProvider {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        if self.location.is_none() {
            return Err("Latitude and longitude must be set.".to_string());
        }
        Ok(())
    }

    fn get_location(&mut self) -> Result<Location, String> {
        self.location
            .ok_or_else(|| "Location not set".to_string())
    }

    fn name(&self) -> &str {
        "manual"
    }

    fn print_help(&self) {
        println!("Specify location manually.");
        println!();
        println!("  lat=N\t\tLatitude");
        println!("  lon=N\t\tLongitude");
        println!();
        println!("Both values are expected to be floating point numbers,");
        println!("negative values representing west / south, respectively.");
        println!();
    }

    fn set_option(&mut self, key: &str, value: &str) -> Result<(), String> {
        let v: f32 = value
            .parse()
            .map_err(|_| format!("Malformed argument: {}", value))?;

        match key.to_lowercase().as_str() {
            "lat" => {
                let mut loc = self.location.unwrap_or(Location { lat: 0.0, lon: 0.0 });
                loc.lat = v;
                self.location = Some(loc);
                Ok(())
            }
            "lon" => {
                let mut loc = self.location.unwrap_or(Location { lat: 0.0, lon: 0.0 });
                loc.lon = v;
                self.location = Some(loc);
                Ok(())
            }
            _ => Err(format!("Unknown method parameter: `{}`", key)),
        }
    }
}
