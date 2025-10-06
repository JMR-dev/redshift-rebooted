/// City database for interactive location selection
/// Contains major cities organized by country

use crate::types::Location;

#[derive(Debug, Clone)]
pub struct City {
    pub name: &'static str,
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Clone)]
pub struct Country {
    pub name: &'static str,
    pub cities: &'static [City],
}

// Major cities database organized by country
pub const COUNTRIES: &[Country] = &[
    Country {
        name: "United States",
        cities: &[
            City { name: "New York, NY", lat: 40.7128, lon: -74.0060 },
            City { name: "Los Angeles, CA", lat: 34.0522, lon: -118.2437 },
            City { name: "Chicago, IL", lat: 41.8781, lon: -87.6298 },
            City { name: "Houston, TX", lat: 29.7604, lon: -95.3698 },
            City { name: "Phoenix, AZ", lat: 33.4484, lon: -112.0740 },
            City { name: "Philadelphia, PA", lat: 39.9526, lon: -75.1652 },
            City { name: "San Antonio, TX", lat: 29.4241, lon: -98.4936 },
            City { name: "San Diego, CA", lat: 32.7157, lon: -117.1611 },
            City { name: "Dallas, TX", lat: 32.7767, lon: -96.7970 },
            City { name: "San Jose, CA", lat: 37.3382, lon: -121.8863 },
            City { name: "Austin, TX", lat: 30.2672, lon: -97.7431 },
            City { name: "Jacksonville, FL", lat: 30.3322, lon: -81.6557 },
            City { name: "Fort Worth, TX", lat: 32.7555, lon: -97.3308 },
            City { name: "Columbus, OH", lat: 39.9612, lon: -82.9988 },
            City { name: "Charlotte, NC", lat: 35.2271, lon: -80.8431 },
            City { name: "San Francisco, CA", lat: 37.7749, lon: -122.4194 },
            City { name: "Indianapolis, IN", lat: 39.7684, lon: -86.1581 },
            City { name: "Seattle, WA", lat: 47.6062, lon: -122.3321 },
            City { name: "Denver, CO", lat: 39.7392, lon: -104.9903 },
            City { name: "Boston, MA", lat: 42.3601, lon: -71.0589 },
            City { name: "Portland, OR", lat: 45.5152, lon: -122.6784 },
            City { name: "Miami, FL", lat: 25.7617, lon: -80.1918 },
            City { name: "Atlanta, GA", lat: 33.7490, lon: -84.3880 },
            City { name: "Las Vegas, NV", lat: 36.1699, lon: -115.1398 },
        ],
    },
    Country {
        name: "Canada",
        cities: &[
            City { name: "Toronto, ON", lat: 43.6532, lon: -79.3832 },
            City { name: "Montreal, QC", lat: 45.5017, lon: -73.5673 },
            City { name: "Vancouver, BC", lat: 49.2827, lon: -123.1207 },
            City { name: "Calgary, AB", lat: 51.0447, lon: -114.0719 },
            City { name: "Edmonton, AB", lat: 53.5461, lon: -113.4938 },
            City { name: "Ottawa, ON", lat: 45.4215, lon: -75.6972 },
            City { name: "Winnipeg, MB", lat: 49.8951, lon: -97.1384 },
            City { name: "Quebec City, QC", lat: 46.8139, lon: -71.2080 },
        ],
    },
    Country {
        name: "United Kingdom",
        cities: &[
            City { name: "London", lat: 51.5074, lon: -0.1278 },
            City { name: "Manchester", lat: 53.4808, lon: -2.2426 },
            City { name: "Birmingham", lat: 52.4862, lon: -1.8904 },
            City { name: "Leeds", lat: 53.8008, lon: -1.5491 },
            City { name: "Glasgow", lat: 55.8642, lon: -4.2518 },
            City { name: "Edinburgh", lat: 55.9533, lon: -3.1883 },
            City { name: "Liverpool", lat: 53.4084, lon: -2.9916 },
            City { name: "Bristol", lat: 51.4545, lon: -2.5879 },
        ],
    },
    Country {
        name: "Germany",
        cities: &[
            City { name: "Berlin", lat: 52.5200, lon: 13.4050 },
            City { name: "Hamburg", lat: 53.5511, lon: 9.9937 },
            City { name: "Munich", lat: 48.1351, lon: 11.5820 },
            City { name: "Cologne", lat: 50.9375, lon: 6.9603 },
            City { name: "Frankfurt", lat: 50.1109, lon: 8.6821 },
            City { name: "Stuttgart", lat: 48.7758, lon: 9.1829 },
            City { name: "Düsseldorf", lat: 51.2277, lon: 6.7735 },
            City { name: "Dortmund", lat: 51.5136, lon: 7.4653 },
        ],
    },
    Country {
        name: "France",
        cities: &[
            City { name: "Paris", lat: 48.8566, lon: 2.3522 },
            City { name: "Marseille", lat: 43.2965, lon: 5.3698 },
            City { name: "Lyon", lat: 45.7640, lon: 4.8357 },
            City { name: "Toulouse", lat: 43.6047, lon: 1.4442 },
            City { name: "Nice", lat: 43.7102, lon: 7.2620 },
            City { name: "Nantes", lat: 47.2184, lon: -1.5536 },
            City { name: "Strasbourg", lat: 48.5734, lon: 7.7521 },
            City { name: "Bordeaux", lat: 44.8378, lon: -0.5792 },
        ],
    },
    Country {
        name: "Spain",
        cities: &[
            City { name: "Madrid", lat: 40.4168, lon: -3.7038 },
            City { name: "Barcelona", lat: 41.3851, lon: 2.1734 },
            City { name: "Valencia", lat: 39.4699, lon: -0.3763 },
            City { name: "Seville", lat: 37.3891, lon: -5.9845 },
            City { name: "Zaragoza", lat: 41.6488, lon: -0.8891 },
            City { name: "Málaga", lat: 36.7213, lon: -4.4214 },
            City { name: "Bilbao", lat: 43.2630, lon: -2.9350 },
        ],
    },
    Country {
        name: "Italy",
        cities: &[
            City { name: "Rome", lat: 41.9028, lon: 12.4964 },
            City { name: "Milan", lat: 45.4642, lon: 9.1900 },
            City { name: "Naples", lat: 40.8518, lon: 14.2681 },
            City { name: "Turin", lat: 45.0703, lon: 7.6869 },
            City { name: "Palermo", lat: 38.1157, lon: 13.3615 },
            City { name: "Florence", lat: 43.7696, lon: 11.2558 },
            City { name: "Venice", lat: 45.4408, lon: 12.3155 },
        ],
    },
    Country {
        name: "Japan",
        cities: &[
            City { name: "Tokyo", lat: 35.6762, lon: 139.6503 },
            City { name: "Osaka", lat: 34.6937, lon: 135.5023 },
            City { name: "Yokohama", lat: 35.4437, lon: 139.6380 },
            City { name: "Nagoya", lat: 35.1815, lon: 136.9066 },
            City { name: "Sapporo", lat: 43.0642, lon: 141.3469 },
            City { name: "Fukuoka", lat: 33.5904, lon: 130.4017 },
            City { name: "Kobe", lat: 34.6901, lon: 135.1955 },
            City { name: "Kyoto", lat: 35.0116, lon: 135.7681 },
        ],
    },
    Country {
        name: "China",
        cities: &[
            City { name: "Beijing", lat: 39.9042, lon: 116.4074 },
            City { name: "Shanghai", lat: 31.2304, lon: 121.4737 },
            City { name: "Guangzhou", lat: 23.1291, lon: 113.2644 },
            City { name: "Shenzhen", lat: 22.5431, lon: 114.0579 },
            City { name: "Chengdu", lat: 30.5728, lon: 104.0668 },
            City { name: "Hangzhou", lat: 30.2741, lon: 120.1551 },
            City { name: "Wuhan", lat: 30.5928, lon: 114.3055 },
            City { name: "Xi'an", lat: 34.3416, lon: 108.9398 },
        ],
    },
    Country {
        name: "India",
        cities: &[
            City { name: "Mumbai", lat: 19.0760, lon: 72.8777 },
            City { name: "Delhi", lat: 28.7041, lon: 77.1025 },
            City { name: "Bangalore", lat: 12.9716, lon: 77.5946 },
            City { name: "Hyderabad", lat: 17.3850, lon: 78.4867 },
            City { name: "Chennai", lat: 13.0827, lon: 80.2707 },
            City { name: "Kolkata", lat: 22.5726, lon: 88.3639 },
            City { name: "Pune", lat: 18.5204, lon: 73.8567 },
            City { name: "Ahmedabad", lat: 23.0225, lon: 72.5714 },
        ],
    },
    Country {
        name: "Australia",
        cities: &[
            City { name: "Sydney, NSW", lat: -33.8688, lon: 151.2093 },
            City { name: "Melbourne, VIC", lat: -37.8136, lon: 144.9631 },
            City { name: "Brisbane, QLD", lat: -27.4698, lon: 153.0251 },
            City { name: "Perth, WA", lat: -31.9505, lon: 115.8605 },
            City { name: "Adelaide, SA", lat: -34.9285, lon: 138.6007 },
            City { name: "Canberra, ACT", lat: -35.2809, lon: 149.1300 },
        ],
    },
    Country {
        name: "Brazil",
        cities: &[
            City { name: "São Paulo", lat: -23.5505, lon: -46.6333 },
            City { name: "Rio de Janeiro", lat: -22.9068, lon: -43.1729 },
            City { name: "Brasília", lat: -15.8267, lon: -47.9218 },
            City { name: "Salvador", lat: -12.9714, lon: -38.5014 },
            City { name: "Fortaleza", lat: -3.7172, lon: -38.5433 },
            City { name: "Belo Horizonte", lat: -19.9167, lon: -43.9345 },
        ],
    },
    Country {
        name: "Mexico",
        cities: &[
            City { name: "Mexico City", lat: 19.4326, lon: -99.1332 },
            City { name: "Guadalajara", lat: 20.6597, lon: -103.3496 },
            City { name: "Monterrey", lat: 25.6866, lon: -100.3161 },
            City { name: "Puebla", lat: 19.0414, lon: -98.2063 },
            City { name: "Tijuana", lat: 32.5149, lon: -117.0382 },
            City { name: "Cancún", lat: 21.1619, lon: -86.8515 },
        ],
    },
    Country {
        name: "Russia",
        cities: &[
            City { name: "Moscow", lat: 55.7558, lon: 37.6173 },
            City { name: "Saint Petersburg", lat: 59.9343, lon: 30.3351 },
            City { name: "Novosibirsk", lat: 55.0084, lon: 82.9357 },
            City { name: "Yekaterinburg", lat: 56.8389, lon: 60.6057 },
            City { name: "Kazan", lat: 55.8304, lon: 49.0661 },
            City { name: "Vladivostok", lat: 43.1332, lon: 131.9113 },
        ],
    },
    Country {
        name: "South Africa",
        cities: &[
            City { name: "Johannesburg", lat: -26.2041, lon: 28.0473 },
            City { name: "Cape Town", lat: -33.9249, lon: 18.4241 },
            City { name: "Durban", lat: -29.8587, lon: 31.0218 },
            City { name: "Pretoria", lat: -25.7479, lon: 28.2293 },
        ],
    },
    Country {
        name: "Argentina",
        cities: &[
            City { name: "Buenos Aires", lat: -34.6037, lon: -58.3816 },
            City { name: "Córdoba", lat: -31.4201, lon: -64.1888 },
            City { name: "Rosario", lat: -32.9442, lon: -60.6505 },
            City { name: "Mendoza", lat: -32.8895, lon: -68.8458 },
        ],
    },
];

impl City {
    pub fn to_location(&self) -> Location {
        Location {
            lat: self.lat,
            lon: self.lon,
        }
    }
}

/// Search for a city by name (case-insensitive)
pub fn find_city(name: &str) -> Option<(usize, usize)> {
    let name_lower = name.to_lowercase();
    for (country_idx, country) in COUNTRIES.iter().enumerate() {
        for (city_idx, city) in country.cities.iter().enumerate() {
            if city.name.to_lowercase().contains(&name_lower) {
                return Some((country_idx, city_idx));
            }
        }
    }
    None
}

/// Get total number of cities across all countries
pub fn total_cities() -> usize {
    COUNTRIES.iter().map(|c| c.cities.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_countries_not_empty() {
        assert!(!COUNTRIES.is_empty());
    }

    #[test]
    fn test_all_countries_have_cities() {
        for country in COUNTRIES {
            assert!(!country.cities.is_empty(), "{} has no cities", country.name);
        }
    }

    #[test]
    fn test_find_city() {
        assert!(find_city("London").is_some());
        assert!(find_city("Tokyo").is_some());
        assert!(find_city("New York").is_some());
        assert!(find_city("NonexistentCity").is_none());
    }

    #[test]
    fn test_city_to_location() {
        let city = City {
            name: "Test City",
            lat: 40.0,
            lon: -74.0,
        };
        let location = city.to_location();
        assert_eq!(location.lat, 40.0);
        assert_eq!(location.lon, -74.0);
    }

    #[test]
    fn test_total_cities() {
        let count = total_cities();
        assert!(count > 100, "Should have over 100 cities");
    }
}
