mod colorramp;
mod gamma;
mod location;
mod solar;
mod types;

use clap::{Parser, ValueEnum};
use gamma::{DummyGammaMethod, GammaMethod};
use location::{LocationProvider, ManualLocationProvider};
use std::time::{SystemTime, UNIX_EPOCH};
use types::*;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GammaMethodChoice {
    Dummy,
}

#[derive(Parser, Debug)]
#[command(name = "redshift")]
#[command(about = "Adjusts screen color temperature", long_about = None)]
struct Args {
    /// Location as LAT:LON (e.g., 40.7:-74.0)
    #[arg(short, long, value_name = "LAT:LON")]
    location: Option<String>,

    /// Gamma adjustment method
    #[arg(short = 'm', long, default_value = "dummy")]
    method: GammaMethodChoice,

    /// One-shot mode (set temperature and exit)
    #[arg(short = 'o', long)]
    one_shot: bool,

    /// Print mode (display settings and exit)
    #[arg(short = 'p', long)]
    print: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Day temperature (default: 6500K)
    #[arg(short = 't', long, default_value = "6500")]
    temp_day: i32,

    /// Night temperature (default: 3500K)
    #[arg(long, default_value = "3500")]
    temp_night: i32,
}

fn parse_location(loc_str: &str) -> Result<Location, String> {
    let parts: Vec<&str> = loc_str.split(':').collect();
    if parts.len() != 2 {
        return Err("Location must be in format LAT:LON".to_string());
    }

    let lat: f32 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid latitude: {}", parts[0]))?;
    let lon: f32 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid longitude: {}", parts[1]))?;

    if lat < MIN_LAT || lat > MAX_LAT {
        return Err(format!(
            "Latitude must be between {} and {}",
            MIN_LAT, MAX_LAT
        ));
    }
    if lon < MIN_LON || lon > MAX_LON {
        return Err(format!(
            "Longitude must be between {} and {}",
            MIN_LON, MAX_LON
        ));
    }

    Ok(Location { lat, lon })
}

fn get_current_period(
    location: &Location,
    scheme: &TransitionScheme,
) -> (Period, ColorSetting) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let elevation = solar::solar_elevation(now, location.lat as f64, location.lon as f64);

    if elevation >= scheme.high {
        (Period::Daytime, scheme.day)
    } else if elevation <= scheme.low {
        (Period::Night, scheme.night)
    } else {
        (Period::Transition, interpolate_color_setting(
            elevation,
            scheme.low,
            scheme.high,
            &scheme.night,
            &scheme.day,
        ))
    }
}

fn interpolate_color_setting(
    elevation: f64,
    low: f64,
    high: f64,
    night: &ColorSetting,
    day: &ColorSetting,
) -> ColorSetting {
    let alpha = ((elevation - low) / (high - low)) as f32;
    let alpha = alpha.max(0.0).min(1.0);

    ColorSetting {
        temperature: ((1.0 - alpha) * (night.temperature as f32) + alpha * (day.temperature as f32))
            as i32,
        gamma: [
            (1.0 - alpha) * night.gamma[0] + alpha * day.gamma[0],
            (1.0 - alpha) * night.gamma[1] + alpha * day.gamma[1],
            (1.0 - alpha) * night.gamma[2] + alpha * day.gamma[2],
        ],
        brightness: (1.0 - alpha) * night.brightness + alpha * day.brightness,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    /* Validate temperature bounds */
    if args.temp_day < MIN_TEMP || args.temp_day > MAX_TEMP {
        eprintln!(
            "Temperature must be between {} and {}",
            MIN_TEMP, MAX_TEMP
        );
        std::process::exit(1);
    }
    if args.temp_night < MIN_TEMP || args.temp_night > MAX_TEMP {
        eprintln!(
            "Temperature must be between {} and {}",
            MIN_TEMP, MAX_TEMP
        );
        std::process::exit(1);
    }

    /* Set up location provider */
    let mut location_provider: Box<dyn LocationProvider> = if let Some(loc_str) = &args.location {
        let loc = parse_location(loc_str)?;
        Box::new(ManualLocationProvider::with_location(loc.lat, loc.lon))
    } else {
        eprintln!("Location must be specified with -l LAT:LON");
        std::process::exit(1);
    };

    location_provider.init()?;
    location_provider.start()?;
    let location = location_provider.get_location()?;

    if args.verbose {
        println!("Location: {:.2}, {:.2}", location.lat, location.lon);
    }

    /* Set up gamma method */
    let mut gamma_method: Box<dyn GammaMethod> = match args.method {
        GammaMethodChoice::Dummy => Box::new(DummyGammaMethod::new()),
    };

    gamma_method.init()?;
    gamma_method.start()?;

    /* Create transition scheme */
    let mut scheme = TransitionScheme::default();
    scheme.day.temperature = args.temp_day;
    scheme.night.temperature = args.temp_night;

    /* Get current period and color setting */
    let (period, color_setting) = get_current_period(&location, &scheme);

    if args.print {
        println!("Period: {}", period.name());
        println!("Color temperature: {}K", color_setting.temperature);
        println!(
            "Brightness: {:.2}",
            color_setting.brightness
        );
        println!(
            "Gamma: {:.2}, {:.2}, {:.2}",
            color_setting.gamma[0], color_setting.gamma[1], color_setting.gamma[2]
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let elevation = solar::solar_elevation(now, location.lat as f64, location.lon as f64);
        println!("Solar elevation: {:.2}°", elevation);

        return Ok(());
    }

    /* Apply color temperature */
    if args.verbose {
        println!("Period: {}", period.name());
    }

    gamma_method.set_temperature(&color_setting, false)?;

    if args.one_shot {
        return Ok(());
    }

    /* For continual mode, we would loop here updating the temperature
       For now, we just set it once */
    println!("Continual mode not yet implemented. Use -o for one-shot mode.");

    Ok(())
}
