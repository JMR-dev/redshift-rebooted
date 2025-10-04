/// Solar position calculations
/// Ported from legacy/src/solar.c
/// Based on equations from "Astronomical Algorithms" by Jean Meeus
/// Originally from U.S. Department of Commerce, NOAA

use std::f64::consts::PI;

/// Model of atmospheric refraction near horizon (in degrees)
pub const SOLAR_ATM_REFRAC: f64 = 0.833;

pub const SOLAR_ASTRO_TWILIGHT_ELEV: f64 = -18.0;
pub const SOLAR_NAUT_TWILIGHT_ELEV: f64 = -12.0;
pub const SOLAR_CIVIL_TWILIGHT_ELEV: f64 = -6.0;
pub const SOLAR_DAYTIME_ELEV: f64 = 0.0 - SOLAR_ATM_REFRAC;

#[derive(Debug, Clone, Copy)]
pub enum SolarTime {
    Noon,
    Midnight,
    AstroDawn,
    NautDawn,
    CivilDawn,
    Sunrise,
    Sunset,
    CivilDusk,
    NautDusk,
    AstroDusk,
}

impl SolarTime {
    fn angle(&self) -> f64 {
        let angle_deg = match self {
            SolarTime::Noon => 0.0,
            SolarTime::Midnight => 0.0, // Special case handled separately
            SolarTime::AstroDawn => -90.0 + SOLAR_ASTRO_TWILIGHT_ELEV,
            SolarTime::NautDawn => -90.0 + SOLAR_NAUT_TWILIGHT_ELEV,
            SolarTime::CivilDawn => -90.0 + SOLAR_CIVIL_TWILIGHT_ELEV,
            SolarTime::Sunrise => -90.0 + SOLAR_DAYTIME_ELEV,
            SolarTime::Sunset => 90.0 - SOLAR_DAYTIME_ELEV,
            SolarTime::CivilDusk => 90.0 - SOLAR_CIVIL_TWILIGHT_ELEV,
            SolarTime::NautDusk => 90.0 - SOLAR_NAUT_TWILIGHT_ELEV,
            SolarTime::AstroDusk => 90.0 - SOLAR_ASTRO_TWILIGHT_ELEV,
        };
        angle_deg.to_radians()
    }
}

/// Convert radians to degrees
fn deg(x: f64) -> f64 {
    x * (180.0 / PI)
}

/// Convert degrees to radians
fn rad(x: f64) -> f64 {
    x * (PI / 180.0)
}

/// Unix epoch from Julian day
fn epoch_from_jd(jd: f64) -> f64 {
    86400.0 * (jd - 2440587.5)
}

/// Julian day from unix epoch
fn jd_from_epoch(t: f64) -> f64 {
    (t / 86400.0) + 2440587.5
}

/// Julian centuries since J2000.0 from Julian day
fn jcent_from_jd(jd: f64) -> f64 {
    (jd - 2451545.0) / 36525.0
}

/// Julian day from Julian centuries since J2000.0
fn jd_from_jcent(t: f64) -> f64 {
    36525.0 * t + 2451545.0
}

/// Geometric mean longitude of the sun
/// t: Julian centuries since J2000.0
/// Returns: Geometric mean longitude in radians
fn sun_geom_mean_lon(t: f64) -> f64 {
    rad((280.46646 + t * (36000.76983 + t * 0.0003032)).rem_euclid(360.0))
}

/// Geometric mean anomaly of the sun
/// t: Julian centuries since J2000.0
/// Returns: Geometric mean anomaly in radians
fn sun_geom_mean_anomaly(t: f64) -> f64 {
    rad(357.52911 + t * (35999.05029 - t * 0.0001537))
}

/// Eccentricity of earth orbit
/// t: Julian centuries since J2000.0
/// Returns: Eccentricity (unitless)
fn earth_orbit_eccentricity(t: f64) -> f64 {
    0.016708634 - t * (0.000042037 + t * 0.0000001267)
}

/// Equation of center of the sun
/// t: Julian centuries since J2000.0
/// Returns: Equation of center in radians
fn sun_equation_of_center(t: f64) -> f64 {
    let m = sun_geom_mean_anomaly(t);
    let c = m.sin() * (1.914602 - t * (0.004817 + 0.000014 * t))
        + (2.0 * m).sin() * (0.019993 - 0.000101 * t)
        + (3.0 * m).sin() * 0.000289;
    rad(c)
}

/// True longitude of the sun
/// t: Julian centuries since J2000.0
/// Returns: True longitude in radians
fn sun_true_lon(t: f64) -> f64 {
    sun_geom_mean_lon(t) + sun_equation_of_center(t)
}

/// Apparent longitude of the sun
/// t: Julian centuries since J2000.0
/// Returns: Apparent longitude in radians
fn sun_apparent_lon(t: f64) -> f64 {
    let o = sun_true_lon(t);
    let omega = rad(125.04 - 1934.136 * t);
    o - rad(0.00569) - rad(0.00478) * omega.sin()
}

/// Mean obliquity of the ecliptic
/// t: Julian centuries since J2000.0
/// Returns: Mean obliquity in radians
fn mean_ecliptic_obliquity(t: f64) -> f64 {
    let sec = 21.448 - t * (46.815 + t * (0.00059 - t * 0.001813));
    rad(23.0 + (26.0 + (sec / 60.0)) / 60.0)
}

/// Corrected obliquity of the ecliptic
/// t: Julian centuries since J2000.0
/// Returns: Corrected obliquity in radians
fn obliquity_corrected(t: f64) -> f64 {
    let e0 = mean_ecliptic_obliquity(t);
    let omega = rad(125.04 - 1934.136 * t);
    e0 + rad(0.00256) * omega.cos()
}

/// Declination of the sun
/// t: Julian centuries since J2000.0
/// Returns: Declination in radians
fn sun_declination(t: f64) -> f64 {
    let e = obliquity_corrected(t);
    let lambda = sun_apparent_lon(t);
    (e.sin() * lambda.sin()).asin()
}

/// Difference between true solar time and mean solar time
/// t: Julian centuries since J2000.0
/// Returns: Equation of time in minutes
fn equation_of_time(t: f64) -> f64 {
    let epsilon = obliquity_corrected(t);
    let l0 = sun_geom_mean_lon(t);
    let e = earth_orbit_eccentricity(t);
    let m = sun_geom_mean_anomaly(t);

    let y = (epsilon / 2.0).tan().powi(2);

    let eq_time = y * (2.0 * l0).sin() - 2.0 * e * m.sin()
        + 4.0 * e * y * m.sin() * (2.0 * l0).cos()
        - 0.5 * y * y * (4.0 * l0).sin()
        - 1.25 * e * e * (2.0 * m).sin();

    4.0 * deg(eq_time)
}

/// Hour angle for the given solar elevation
/// lat: Latitude in degrees
/// decl: Solar declination in radians
/// elev: Target elevation in radians
/// Returns: Hour angle in radians
fn hour_angle_from_elevation(lat: f64, decl: f64, elev: f64) -> f64 {
    let lat_rad = rad(lat);
    let ha = ((elev.cos() / (lat_rad.cos() * decl.cos()))
        - lat_rad.tan() * decl.tan())
    .acos();
    ha
}

/// Calculate solar elevation at a given time and location
/// date: Unix timestamp
/// lat: Latitude in degrees
/// lon: Longitude in degrees
/// Returns: Solar elevation in degrees
pub fn solar_elevation(date: f64, lat: f64, lon: f64) -> f64 {
    let jd = jd_from_epoch(date);
    let t = jcent_from_jd(jd);

    let decl = sun_declination(t);
    let time_offset = equation_of_time(t) + 4.0 * lon;

    let time = (date.rem_euclid(86400.0)) / 60.0 - time_offset;
    let ha = rad((time - 720.0) / 4.0);

    let lat_rad = rad(lat);
    let el = (lat_rad.sin() * decl.sin() + lat_rad.cos() * decl.cos() * ha.cos()).asin();

    deg(el)
}

/// Fill a table with solar event times for the day
/// date: Unix timestamp for the day
/// lat: Latitude in degrees
/// lon: Longitude in degrees
/// Returns: Array of unix timestamps for each solar event
pub fn solar_table_fill(date: f64, lat: f64, lon: f64) -> [f64; 10] {
    let jd = jd_from_epoch(date);
    let t = jcent_from_jd(jd);

    let decl = sun_declination(t);
    let eqtime = equation_of_time(t);

    let mut table = [0.0; 10];

    // Noon
    table[SolarTime::Noon as usize] =
        epoch_from_jd(jd_from_jcent(t)) + (720.0 - 4.0 * lon - eqtime) * 60.0;

    // Midnight
    table[SolarTime::Midnight as usize] = table[SolarTime::Noon as usize] + 43200.0;

    // Calculate times for each elevation-based event
    let events = [
        (SolarTime::AstroDawn, true),
        (SolarTime::NautDawn, true),
        (SolarTime::CivilDawn, true),
        (SolarTime::Sunrise, true),
        (SolarTime::Sunset, false),
        (SolarTime::CivilDusk, false),
        (SolarTime::NautDusk, false),
        (SolarTime::AstroDusk, false),
    ];

    for (event, is_morning) in events {
        let angle = event.angle();
        let ha = hour_angle_from_elevation(lat, decl, angle);

        if ha.is_nan() {
            // Sun never reaches this elevation
            table[event as usize] = f64::NAN;
        } else {
            let ha_deg = deg(ha);
            let offset = if is_morning {
                720.0 - ha_deg * 4.0 - 4.0 * lon - eqtime
            } else {
                720.0 + ha_deg * 4.0 - 4.0 * lon - eqtime
            };
            table[event as usize] = epoch_from_jd(jd_from_jcent(t)) + offset * 60.0;
        }
    }

    table
}
