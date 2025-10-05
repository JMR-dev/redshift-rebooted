mod colorramp;
mod gamma;
mod gamma_randr;
mod location;
mod solar;
mod types;

use clap::{Parser, ValueEnum};
use gamma::{DummyGammaMethod, GammaMethod};
use gamma_randr::RandrGammaMethod;
use location::{LocationProvider, ManualLocationProvider};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use types::*;

/* Duration of sleep between screen updates (milliseconds). */
const SLEEP_DURATION: u64 = 5000;
const SLEEP_DURATION_SHORT: u64 = 100;

/* Length of fade in numbers of short sleep durations. */
const FADE_LENGTH: i32 = 40;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GammaMethodChoice {
    Randr,
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
    #[arg(short = 'm', long, default_value = "randr")]
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

/* Determine how far through the transition we are based on elevation.
   Returns a value from 0.0 (night) to 1.0 (day). */
fn get_transition_progress_from_elevation(scheme: &TransitionScheme, elevation: f64) -> f64 {
    if elevation < scheme.low {
        0.0
    } else if elevation < scheme.high {
        (scheme.low - elevation) / (scheme.low - scheme.high)
    } else {
        1.0
    }
}

/* Use transition progress to interpolate color settings.
   Progress from 0.0 (night) to 1.0 (day). */
fn interpolate_transition_scheme(
    scheme: &TransitionScheme,
    progress: f64,
    result: &mut ColorSetting,
) {
    let alpha = progress.max(0.0).min(1.0);

    result.temperature = ((1.0 - alpha) * (scheme.night.temperature as f64)
        + alpha * (scheme.day.temperature as f64)) as i32;
    result.brightness = ((1.0 - alpha) * (scheme.night.brightness as f64)
        + alpha * (scheme.day.brightness as f64)) as f32;
    result.gamma[0] = ((1.0 - alpha) * (scheme.night.gamma[0] as f64)
        + alpha * (scheme.day.gamma[0] as f64)) as f32;
    result.gamma[1] = ((1.0 - alpha) * (scheme.night.gamma[1] as f64)
        + alpha * (scheme.day.gamma[1] as f64)) as f32;
    result.gamma[2] = ((1.0 - alpha) * (scheme.night.gamma[2] as f64)
        + alpha * (scheme.day.gamma[2] as f64)) as f32;
}

/* Return true if color settings have major differences.
   Used to determine if a fade should be applied in continual mode. */
fn color_setting_diff_is_major(first: &ColorSetting, second: &ColorSetting) -> bool {
    (first.temperature - second.temperature).abs() > 25
        || (first.brightness - second.brightness).abs() > 0.1
        || (first.gamma[0] - second.gamma[0]).abs() > 0.1
        || (first.gamma[1] - second.gamma[1]).abs() > 0.1
        || (first.gamma[2] - second.gamma[2]).abs() > 0.1
}

/* Interpolate between two color settings using alpha (0.0 to 1.0). */
fn interpolate_color_settings(
    first: &ColorSetting,
    second: &ColorSetting,
    alpha: f64,
    result: &mut ColorSetting,
) {
    let alpha = alpha.max(0.0).min(1.0);

    result.temperature = ((1.0 - alpha) * (first.temperature as f64)
        + alpha * (second.temperature as f64)) as i32;
    result.brightness = ((1.0 - alpha) * (first.brightness as f64)
        + alpha * (second.brightness as f64)) as f32;
    result.gamma[0] = ((1.0 - alpha) * (first.gamma[0] as f64)
        + alpha * (second.gamma[0] as f64)) as f32;
    result.gamma[1] = ((1.0 - alpha) * (first.gamma[1] as f64)
        + alpha * (second.gamma[1] as f64)) as f32;
    result.gamma[2] = ((1.0 - alpha) * (first.gamma[2] as f64)
        + alpha * (second.gamma[2] as f64)) as f32;
}

/* Ease fade function - cubic interpolation for smooth transitions. */
fn ease_fade(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
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
        GammaMethodChoice::Randr => Box::new(RandrGammaMethod::new()),
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

    /* Continual mode - continuously adjust color temperature */
    run_continual_mode(&location, &scheme, gamma_method.as_mut(), args.verbose)?;

    Ok(())
}

/* Run continual mode loop.
   This is the main loop of the continual mode which keeps track of the
   current time and continuously updates the screen to the appropriate
   color temperature. */
fn run_continual_mode(
    location: &Location,
    scheme: &TransitionScheme,
    gamma_method: &mut dyn GammaMethod,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    /* Fade parameters */
    let mut fade_length: i32 = 0;
    let mut fade_time: i32 = 0;
    let mut fade_start_interp = ColorSetting::default();

    /* Save previous parameters so we can avoid printing status updates if
       the values did not change. */
    let mut prev_period = Period::None;
    let mut prev_target_interp = ColorSetting::default();
    let mut interp = ColorSetting::default();

    if verbose {
        println!("Color temperature: {}K", interp.temperature);
        println!("Brightness: {:.2}", interp.brightness);
    }

    /* Continuously adjust color temperature */
    loop {
        /* Get current time */
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        /* Current angular elevation of the sun */
        let elevation = solar::solar_elevation(now, location.lat as f64, location.lon as f64);

        /* Determine period and transition progress */
        let period = if elevation >= scheme.high {
            Period::Daytime
        } else if elevation <= scheme.low {
            Period::Night
        } else {
            Period::Transition
        };

        let transition_prog = get_transition_progress_from_elevation(scheme, elevation);

        /* Use transition progress to get target color temperature */
        let mut target_interp = ColorSetting::default();
        interpolate_transition_scheme(scheme, transition_prog, &mut target_interp);

        /* Print period if it changed during this update,
           or if we are in the transition period. In transition we
           print the progress, so we always print it in that case. */
        if verbose && (period != prev_period || period == Period::Transition) {
            match period {
                Period::Transition => {
                    println!("Period: Transition ({:.1}%)", transition_prog * 100.0);
                }
                _ => {
                    println!("Period: {}", period.name());
                }
            }
        }

        /* Start fade if the parameter differences are too big to apply instantly. */
        if (fade_length == 0 && color_setting_diff_is_major(&interp, &target_interp))
            || (fade_length != 0 && color_setting_diff_is_major(&target_interp, &prev_target_interp))
        {
            fade_length = FADE_LENGTH;
            fade_time = 0;
            fade_start_interp = interp;
        }

        /* Handle ongoing fade */
        if fade_length != 0 {
            fade_time += 1;
            let frac = fade_time as f64 / fade_length as f64;
            let alpha = ease_fade(frac).max(0.0).min(1.0);

            interpolate_color_settings(&fade_start_interp, &target_interp, alpha, &mut interp);

            if fade_time > fade_length {
                fade_time = 0;
                fade_length = 0;
            }
        } else {
            interp = target_interp;
        }

        if verbose {
            if prev_target_interp.temperature != target_interp.temperature {
                println!("Color temperature: {}K", target_interp.temperature);
            }
            if prev_target_interp.brightness != target_interp.brightness {
                println!("Brightness: {:.2}", target_interp.brightness);
            }
        }

        /* Adjust temperature */
        gamma_method.set_temperature(&interp, false)?;

        /* Save period and target color setting as previous */
        prev_period = period;
        prev_target_interp = target_interp;

        /* Sleep length depends on whether a fade is ongoing. */
        let delay = if fade_length != 0 {
            SLEEP_DURATION_SHORT
        } else {
            SLEEP_DURATION
        };

        std::thread::sleep(Duration::from_millis(delay));
    }
}
