/// Interactive location selection UI
/// Allows users to select their location from a list of countries and cities

use crate::cities::{COUNTRIES, City, Country};
use crate::types::Location;
use std::io::{self, Write};

/// Display a numbered list and get user selection
fn get_selection(prompt: &str, items: &[impl std::fmt::Display], max: usize) -> Result<usize, String> {
    println!("\n{}", prompt);
    println!("{}", "=".repeat(prompt.len()));

    for (i, item) in items.iter().take(max).enumerate() {
        println!("{:3}. {}", i + 1, item);
    }

    print!("\nEnter number (1-{}): ", max);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;

    let choice: usize = input
        .trim()
        .parse()
        .map_err(|_| "Invalid number entered".to_string())?;

    if choice < 1 || choice > max {
        return Err(format!("Number must be between 1 and {}", max));
    }

    Ok(choice - 1)
}

impl std::fmt::Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Display for City {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Interactively select location from country/city lists
pub fn select_location_interactive() -> Result<Location, String> {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║          Redshift - Location Selection                   ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("\nAutomatic location detection is not available.");
    println!("Please select your country and nearest major city.\n");

    // Select country
    let country_idx = get_selection(
        "Select your country:",
        COUNTRIES,
        COUNTRIES.len(),
    )?;

    let country = &COUNTRIES[country_idx];
    println!("\nSelected: {}", country.name);

    // Select city
    let city_idx = get_selection(
        &format!("Select your nearest city in {}:", country.name),
        country.cities,
        country.cities.len(),
    )?;

    let city = &country.cities[city_idx];
    println!("\nSelected: {}", city.name);
    println!("Location: {:.4}°, {:.4}°", city.lat, city.lon);

    Ok(city.to_location())
}

/// Search for a city by name
pub fn search_city_interactive() -> Result<Location, String> {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║          Redshift - City Search                          ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    print!("\nEnter city name to search: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;

    let search_term = input.trim();

    // Find matching cities
    let mut matches: Vec<(usize, usize)> = Vec::new();
    let search_lower = search_term.to_lowercase();

    for (country_idx, country) in COUNTRIES.iter().enumerate() {
        for (city_idx, city) in country.cities.iter().enumerate() {
            if city.name.to_lowercase().contains(&search_lower) {
                matches.push((country_idx, city_idx));
            }
        }
    }

    if matches.is_empty() {
        return Err(format!("No cities found matching '{}'", search_term));
    }

    if matches.len() == 1 {
        let (country_idx, city_idx) = matches[0];
        let city = &COUNTRIES[country_idx].cities[city_idx];
        println!("\nFound: {} ({})", city.name, COUNTRIES[country_idx].name);
        println!("Location: {:.4}°, {:.4}°", city.lat, city.lon);
        return Ok(city.to_location());
    }

    // Multiple matches - let user choose
    println!("\nFound {} matching cities:", matches.len());
    println!("{}", "=".repeat(40));

    for (i, &(country_idx, city_idx)) in matches.iter().enumerate() {
        let city = &COUNTRIES[country_idx].cities[city_idx];
        let country = &COUNTRIES[country_idx];
        println!("{:3}. {} ({})", i + 1, city.name, country.name);
    }

    print!("\nEnter number (1-{}): ", matches.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;

    let choice: usize = input
        .trim()
        .parse()
        .map_err(|_| "Invalid number entered".to_string())?;

    if choice < 1 || choice > matches.len() {
        return Err(format!("Number must be between 1 and {}", matches.len()));
    }

    let (country_idx, city_idx) = matches[choice - 1];
    let city = &COUNTRIES[country_idx].cities[city_idx];
    println!("\nSelected: {}", city.name);
    println!("Location: {:.4}°, {:.4}°", city.lat, city.lon);

    Ok(city.to_location())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_display() {
        let country = &COUNTRIES[0];
        let display = format!("{}", country);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_city_display() {
        let city = &COUNTRIES[0].cities[0];
        let display = format!("{}", city);
        assert!(!display.is_empty());
    }
}
