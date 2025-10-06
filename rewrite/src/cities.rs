/// City database for interactive location selection
/// Contains major cities organized by country

use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct City {
    pub city: String,
    pub city_ascii: String,
    pub lat: String,
    pub lng: String,
    pub country: String,
    pub admin_name: String,
    pub population: String,
    pub id: String,
    #[serde(rename = "isCityCountry")]
    pub is_city_country: bool,
}

impl City {
    /// Get display name for the city based on UTF-8 locale support
    pub fn display_name(&self) -> String {
        if is_utf8_locale() && self.city != self.city_ascii {
            format!("{} ({})", self.city, self.city_ascii)
        } else {
            self.city_ascii.clone()
        }
    }

    /// Parse latitude as f64
    pub fn latitude(&self) -> Result<f64, std::num::ParseFloatError> {
        self.lat.parse()
    }

    /// Parse longitude as f64
    pub fn longitude(&self) -> Result<f64, std::num::ParseFloatError> {
        self.lng.parse()
    }
}

lazy_static! {
    /// Global hash map of countries to cities, loaded at first access
    pub static ref CITIES_BY_COUNTRY: HashMap<String, Vec<City>> = {
        let json_data = include_str!("../data/filtered_world_cities.json");
        let cities: Vec<City> = serde_json::from_str(json_data)
            .expect("Failed to parse filtered_world_cities.json");

        let mut map: HashMap<String, Vec<City>> = HashMap::new();
        for city in cities {
            map.entry(city.country.clone())
                .or_insert_with(Vec::new)
                .push(city);
        }
        map
    };
}

/// Check if the current locale supports UTF-8
fn is_utf8_locale() -> bool {
    env::var("LANG")
        .ok()
        .map(|lang| lang.to_uppercase().contains("UTF-8") || lang.to_uppercase().contains("UTF8"))
        .unwrap_or(false)
}

/// Get a sorted list of all country names
pub fn get_countries() -> Vec<String> {
    let mut countries: Vec<String> = CITIES_BY_COUNTRY.keys().cloned().collect();
    countries.sort();
    countries
}

/// Get cities for a specific country
pub fn get_cities_for_country(country: &str) -> Option<&Vec<City>> {
    CITIES_BY_COUNTRY.get(country)
}

/// Interactive country selection
pub fn select_country() -> Result<String, dialoguer::Error> {
    use dialoguer::Select;

    let countries = get_countries();
    let selection = Select::new()
        .with_prompt("Select your country")
        .items(&countries)
        .interact()?;

    Ok(countries[selection].clone())
}

/// Interactive city selection for a given country
pub fn select_city(country: &str) -> Result<City, String> {
    use dialoguer::Select;

    let cities = get_cities_for_country(country)
        .ok_or_else(|| format!("No cities found for country: {}", country))?;

    // If it's a city-state, auto-select the single city
    if cities.len() == 1 && cities[0].is_city_country {
        return Ok(cities[0].clone());
    }

    // Prepare display names
    let display_names: Vec<String> = cities.iter()
        .map(|c| c.display_name())
        .collect();

    let selection = Select::new()
        .with_prompt("Select your city")
        .items(&display_names)
        .interact()
        .map_err(|e| format!("Selection failed: {}", e))?;

    Ok(cities[selection].clone())
}

/// Interactive location selection - select country then city
pub fn select_location() -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let country = select_country()?;
    let city = select_city(&country)?;

    let lat = city.latitude()?;
    let lon = city.longitude()?;

    println!("Selected: {} - {}", country, city.display_name());
    println!("Coordinates: {:.4}, {:.4}", lat, lon);

    Ok((lat, lon))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cities_loaded() {
        assert!(!CITIES_BY_COUNTRY.is_empty(), "Cities should be loaded");
    }

    #[test]
    fn test_get_countries() {
        let countries = get_countries();
        assert!(!countries.is_empty(), "Should have countries");
        // Verify sorting
        let mut sorted = countries.clone();
        sorted.sort();
        assert_eq!(countries, sorted, "Countries should be sorted");
    }

    #[test]
    fn test_city_display_name() {
        let city = City {
            city: "Āqchah".to_string(),
            city_ascii: "Aqchah".to_string(),
            lat: "36.9114".to_string(),
            lng: "66.1858".to_string(),
            country: "Afghanistan".to_string(),
            admin_name: "Jowzjān".to_string(),
            population: "1012000".to_string(),
            id: "1004364776".to_string(),
            is_city_country: false,
        };

        // Display name format depends on UTF-8 locale, so just ensure it doesn't panic
        let _ = city.display_name();
    }

    #[test]
    fn test_city_coordinates() {
        let city = City {
            city: "Kabul".to_string(),
            city_ascii: "Kabul".to_string(),
            lat: "34.5253".to_string(),
            lng: "69.1783".to_string(),
            country: "Afghanistan".to_string(),
            admin_name: "Kābul".to_string(),
            population: "4273156".to_string(),
            id: "1004993580".to_string(),
            is_city_country: false,
        };

        assert_eq!(city.latitude().unwrap(), 34.5253);
        assert_eq!(city.longitude().unwrap(), 69.1783);
    }
}
