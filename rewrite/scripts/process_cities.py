#!/usr/bin/env python3
"""
Process worldcities.json to filter cities based on:
- Population >= 1 million OR capital city
- If country has < 5 cities with 1M+ population, keep largest cities + capitals
- For city-states (only 1 city), mark for auto-selection
"""

import json
import sys
import os
from collections import defaultdict

# Prompt for input file path
input_path = input("Enter the path to worldcities.json (using Simple Maps data): ").strip()

# Expand user home directory if present
input_path = os.path.expanduser(input_path)

# Check if file exists
if not os.path.isfile(input_path):
    print(f"Error: File not found: {input_path}")
    sys.exit(1)

# Prompt for output directory
output_dir = input("Enter the directory to save filtered_world_cities.json: ").strip()

# Expand user home directory if present
output_dir = os.path.expanduser(output_dir)

# Create output directory if it doesn't exist
if not os.path.exists(output_dir):
    os.makedirs(output_dir)

# Construct output file path
output_path = os.path.join(output_dir, 'filtered_world_cities.json')

# Read the input JSON
print(f"\nReading from: {input_path}")
with open(input_path, 'r') as f:
    cities = json.load(f)

# Group cities by country
cities_by_country = defaultdict(list)
for city in cities:
    cities_by_country[city['country']].append(city)

# Process each country
filtered_cities = []
countries_with_single_city = {}

for country, country_cities in cities_by_country.items():
    # Get cities with population >= 1 million
    million_plus = []
    capitals = []

    for city in country_cities:
        pop = city.get('population')
        is_capital = bool(city.get('capital') and city.get('capital') not in ("", None))

        # Parse population
        try:
            pop_int = int(pop) if pop else 0
        except (ValueError, TypeError):
            pop_int = 0

        city['population_int'] = pop_int

        if is_capital:
            capitals.append(city)

        if pop_int >= 1000000:
            million_plus.append(city)

    # Apply filtering rules
    if len(million_plus) >= 5:
        # Keep cities with 1M+ population and capitals
        selected = million_plus + [c for c in capitals if c not in million_plus]
    else:
        # Keep largest cities + capitals
        # Sort by population
        sorted_cities = sorted(country_cities, key=lambda x: x['population_int'], reverse=True)

        # Take top cities and ensure capitals are included
        num_to_keep = max(5, len(million_plus))
        selected = sorted_cities[:num_to_keep]

        # Add any capitals not already included
        for capital in capitals:
            if capital not in selected:
                selected.append(capital)

    # Remove duplicates
    selected_unique = []
    seen_ids = set()
    for city in selected:
        if city['id'] not in seen_ids:
            selected_unique.append(city)
            seen_ids.add(city['id'])

    # Track city-states (countries with only 1 city in final selection)
    is_city_state = len(selected_unique) == 1
    countries_with_single_city[country] = is_city_state

    filtered_cities.extend(selected_unique)

# Keep only required fields and sort
output_cities = []
for city in filtered_cities:
    output_cities.append({
        'city': city['city'],
        'city_ascii': city['city_ascii'],
        'lat': city['lat'],
        'lng': city['lng'],
        'country': city['country'],
        'admin_name': city['admin_name'],
        'population': city['population'],
        'id': city['id'],
        'isCityCountry': countries_with_single_city.get(city['country'], False)
    })

# Sort by country, then by population
def safe_pop_int(pop_str):
    try:
        return -int(float(pop_str)) if pop_str else 0
    except (ValueError, TypeError):
        return 0

output_cities.sort(key=lambda x: (x['country'], safe_pop_int(x['population'])))

# Write output
print(f"Writing to: {output_path}")
with open(output_path, 'w') as f:
    json.dump(output_cities, f, indent=2, ensure_ascii=False)

city_states_count = sum(1 for v in countries_with_single_city.values() if v)

print(f"\nProcessed {len(cities)} cities")
print(f"Filtered to {len(output_cities)} cities")
print(f"Countries: {len(cities_by_country)}")
print(f"City-states (auto-select): {city_states_count}")
print(f"\nOutput successfully written to: {output_path}")
