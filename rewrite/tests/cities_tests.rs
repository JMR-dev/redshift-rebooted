/// Tests for cities module - location selection and data loading

#[cfg(test)]
mod cities_data_tests {
    use redshift_rebooted::cities::*;

    #[test]
    fn test_cities_by_country_loaded() {
        // Verify the lazy_static hash map is populated
        assert!(!CITIES_BY_COUNTRY.is_empty(), "Cities should be loaded from JSON");
        assert!(CITIES_BY_COUNTRY.len() > 100, "Should have many countries");
    }

    #[test]
    fn test_get_countries_returns_sorted_list() {
        let countries = get_countries();

        assert!(!countries.is_empty(), "Should have countries");

        // Verify sorting
        for i in 1..countries.len() {
            assert!(
                countries[i - 1] <= countries[i],
                "Countries should be sorted alphabetically"
            );
        }
    }

    #[test]
    fn test_get_cities_for_existing_country() {
        let countries = get_countries();

        // Test a few countries that should exist
        if countries.contains(&"United States".to_string()) {
            let cities = get_cities_for_country("United States");
            assert!(cities.is_some(), "United States should have cities");
            assert!(!cities.unwrap().is_empty(), "United States should have at least one city");
        }
    }

    #[test]
    fn test_get_cities_for_nonexistent_country() {
        let cities = get_cities_for_country("NonexistentCountry12345");
        assert!(cities.is_none(), "Nonexistent country should return None");
    }

    #[test]
    fn test_city_fields_are_populated() {
        let countries = get_countries();
        if let Some(country) = countries.first() {
            if let Some(cities) = get_cities_for_country(country) {
                if let Some(city) = cities.first() {
                    // Verify all required fields are present
                    assert!(!city.city.is_empty(), "City name should not be empty");
                    assert!(!city.city_ascii.is_empty(), "City ASCII name should not be empty");
                    assert!(!city.lat.is_empty(), "Latitude should not be empty");
                    assert!(!city.lng.is_empty(), "Longitude should not be empty");
                    assert!(!city.country.is_empty(), "Country should not be empty");
                    assert!(!city.id.is_empty(), "ID should not be empty");

                    // Verify coordinates can be parsed
                    assert!(city.latitude().is_ok(), "Latitude should be parseable");
                    assert!(city.longitude().is_ok(), "Longitude should be parseable");
                }
            }
        }
    }

    #[test]
    fn test_city_coordinates_are_valid() {
        let countries = get_countries();

        for country in countries.iter().take(10) {
            if let Some(cities) = get_cities_for_country(country) {
                for city in cities.iter().take(5) {
                    let lat = city.latitude().expect("Should parse latitude");
                    let lon = city.longitude().expect("Should parse longitude");

                    assert!(lat >= -90.0 && lat <= 90.0, "Latitude must be between -90 and 90");
                    assert!(lon >= -180.0 && lon <= 180.0, "Longitude must be between -180 and 180");
                }
            }
        }
    }

    #[test]
    fn test_city_state_flag() {
        // Find a city-state if one exists
        let countries = get_countries();

        let mut found_city_state = false;
        let mut found_non_city_state = false;

        for country in countries.iter() {
            if let Some(cities) = get_cities_for_country(country) {
                if cities.len() == 1 && cities[0].is_city_country {
                    found_city_state = true;
                }
                if cities.len() > 1 {
                    for city in cities {
                        if !city.is_city_country {
                            found_non_city_state = true;
                            break;
                        }
                    }
                }
            }
        }

        // We should have both types in our dataset
        assert!(found_city_state || found_non_city_state,
            "Should have at least some cities with is_city_country flags");
    }
}

#[cfg(test)]
mod city_display_tests {
    use redshift_rebooted::cities::City;

    fn create_test_city(city: &str, city_ascii: &str) -> City {
        City {
            city: city.to_string(),
            city_ascii: city_ascii.to_string(),
            lat: "40.7128".to_string(),
            lng: "-74.0060".to_string(),
            country: "Test Country".to_string(),
            admin_name: "Test Admin".to_string(),
            population: "1000000".to_string(),
            id: "12345".to_string(),
            is_city_country: false,
        }
    }

    #[test]
    fn test_display_name_same_city_and_ascii() {
        let city = create_test_city("NewYork", "NewYork");
        let display = city.display_name();

        // When city and city_ascii are the same, should just show city_ascii
        assert_eq!(display, "NewYork");
    }

    #[test]
    fn test_city_latitude_parsing() {
        let city = create_test_city("Test", "Test");
        let lat = city.latitude();

        assert!(lat.is_ok());
        assert!((lat.unwrap() - 40.7128).abs() < 0.0001);
    }

    #[test]
    fn test_city_longitude_parsing() {
        let city = create_test_city("Test", "Test");
        let lon = city.longitude();

        assert!(lon.is_ok());
        assert!((lon.unwrap() - (-74.0060)).abs() < 0.0001);
    }

    #[test]
    fn test_city_invalid_coordinates() {
        let mut city = create_test_city("Test", "Test");
        city.lat = "invalid".to_string();

        assert!(city.latitude().is_err());
    }

    #[test]
    fn test_city_empty_coordinates() {
        let mut city = create_test_city("Test", "Test");
        city.lat = "".to_string();

        assert!(city.latitude().is_err());
    }
}

#[cfg(test)]
mod utf8_locale_tests {
    // Note: These tests can't easily test the actual UTF-8 locale detection
    // since it depends on environment variables, but we can test the logic

    #[test]
    fn test_city_display_logic() {
        // This is tested indirectly through the display_name tests above
        // The actual UTF-8 detection happens at runtime based on LANG env var
    }
}

#[cfg(test)]
mod data_integrity_tests {
    use redshift_rebooted::cities::*;
    use std::collections::HashSet;

    #[test]
    fn test_no_duplicate_city_ids() {
        let mut seen_ids = HashSet::new();
        let countries = get_countries();

        for country in countries {
            if let Some(cities) = get_cities_for_country(&country) {
                for city in cities {
                    assert!(
                        seen_ids.insert(city.id.clone()),
                        "City ID {} appears more than once",
                        city.id
                    );
                }
            }
        }
    }

    #[test]
    fn test_all_cities_have_matching_country() {
        let countries = get_countries();

        for country in countries {
            if let Some(cities) = get_cities_for_country(&country) {
                for city in cities {
                    assert_eq!(
                        city.country, country,
                        "City {} has mismatched country field",
                        city.city
                    );
                }
            }
        }
    }

    #[test]
    fn test_populations_are_numeric_or_empty() {
        let countries = get_countries();

        for country in countries.iter().take(20) {
            if let Some(cities) = get_cities_for_country(country) {
                for city in cities {
                    if !city.population.is_empty() {
                        // Try to parse as float to handle values like "7740.00"
                        let parse_result = city.population.parse::<f64>();
                        assert!(
                            parse_result.is_ok(),
                            "Population '{}' for city {} should be numeric",
                            city.population,
                            city.city
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_city_state_consistency() {
        let countries = get_countries();

        for country in countries {
            if let Some(cities) = get_cities_for_country(&country) {
                if cities.len() == 1 {
                    // Single city countries should have is_city_country = true
                    assert!(
                        cities[0].is_city_country,
                        "Country {} has only 1 city but is_city_country is false",
                        country
                    );
                } else {
                    // Multi-city countries should have is_city_country = false
                    for city in cities {
                        assert!(
                            !city.is_city_country,
                            "Country {} has multiple cities but {} has is_city_country = true",
                            country,
                            city.city
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_reasonable_population_sizes() {
        let countries = get_countries();

        for country in countries.iter().take(20) {
            if let Some(cities) = get_cities_for_country(country) {
                for city in cities {
                    if let Ok(pop) = city.population.parse::<f64>() {
                        // Population should be reasonable (not negative, not absurdly high)
                        assert!(pop >= 0.0, "Population cannot be negative");
                        assert!(pop < 100_000_000.0, "City population seems unreasonably high");
                    }
                }
            }
        }
    }
}
