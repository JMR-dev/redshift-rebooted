/// Interactive location selection UI
/// Allows users to select their location from a list of countries and cities

use crate::cities;
use crate::types::Location;

/// Interactively select location from country/city lists
pub fn select_location_interactive() -> Result<Location, String> {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║          Redshift - Location Selection                   ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("\nAutomatic location detection is not available.");
    println!("Please select your country and nearest major city.\n");

    // Select country
    let country = cities::select_country()
        .map_err(|e| format!("Country selection failed: {}", e))?;

    println!("\nSelected: {}", country);

    // Select city
    let city = cities::select_city(&country)?;

    println!("\nSelected: {}", city.display_name());
    println!("Location: {:.4}°, {:.4}°",
        city.latitude().map_err(|e| format!("Invalid latitude: {}", e))?,
        city.longitude().map_err(|e| format!("Invalid longitude: {}", e))?
    );

    Ok(Location {
        lat: city.latitude().map_err(|e| format!("Invalid latitude: {}", e))? as f32,
        lon: city.longitude().map_err(|e| format!("Invalid longitude: {}", e))? as f32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cities_available() {
        let countries = cities::get_countries();
        assert!(!countries.is_empty(), "Should have countries available");
    }

    #[test]
    fn test_cities_for_country() {
        let countries = cities::get_countries();
        if let Some(country) = countries.first() {
            let cities = cities::get_cities_for_country(country);
            assert!(cities.is_some(), "Should have cities for first country");
        }
    }
}
