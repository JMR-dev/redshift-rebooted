mod cities;
mod colorramp;
mod config;
mod config_ini;
mod gamma;
mod gamma_guard;
mod gamma_randr;
mod interactive;
mod location;
mod signals;
mod solar;
mod types;

use clap::{ArgAction, Parser, ValueEnum};
use config::{Config, LocationSource};
use gamma::{DummyGammaMethod, GammaMethod};
use gamma_guard::GammaRestoreGuard;
use gamma_randr::RandrGammaMethod;
use location::{GeoClue2LocationProvider, LocationProvider};
use log::{debug, info, trace};
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
    /// Location as LAT:LON (e.g., 40.7:-74.0), or leave empty for automatic detection
    #[arg(short, long, value_name = "LAT:LON")]
    location: Option<String>,

    /// Disable automatic location (requires manual location)
    #[arg(long)]
    no_auto_location: bool,

    /// Gamma adjustment method
    #[arg(short = 'm', long, default_value = "randr")]
    method: GammaMethodChoice,

    /// One-shot mode (set temperature and exit)
    #[arg(short = 'o', long)]
    one_shot: bool,

    /// Print mode (display settings and exit)
    #[arg(short = 'p', long)]
    print: bool,

    /// Verbose output (can be repeated: -v=info, -vv=debug, -vvv=trace)
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// Day temperature (default: 6500K)
    #[arg(short = 't', long, default_value = "6500")]
    temp_day: i32,

    /// Night temperature (default: 3500K)
    #[arg(long, default_value = "3500")]
    temp_night: i32,

    /// Brightness (day:night or single value)
    #[arg(short = 'b', long)]
    brightness: Option<String>,

    /// Gamma (R:G:B or single value)
    #[arg(short = 'g', long)]
    gamma: Option<String>,
}

impl Args {
    /// Merge with INI config (CLI args take priority)
    fn merge_with_ini(&mut self, ini_config: &config_ini::RedshiftConfig) {
        /* Temperature settings - only use INI if CLI used defaults */
        if self.temp_day == 6500 {
            if let Some(temp) = ini_config.temp_day {
                self.temp_day = temp;
            }
        }
        if self.temp_night == 3500 {
            if let Some(temp) = ini_config.temp_night {
                self.temp_night = temp;
            }
        }

        /* Brightness and gamma - these are new, so always use from INI if not in CLI */
        /* These will be handled separately when building the scheme */
    }
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

/// Determine location using priority system (with INI config support)
fn determine_location_with_ini(
    args: &Args,
    ini_config: &config_ini::RedshiftConfig,
) -> Result<(Location, Config), Box<dyn std::error::Error>> {
    debug!("Determining location using priority system");

    // Priority 1: Command-line argument
    if let Some(loc_str) = &args.location {
        let loc = parse_location(loc_str)?;
        info!("Using location from command-line: {:.4}, {:.4}", loc.lat, loc.lon);

        // Load config for other settings
        let mut config = Config::load().unwrap_or_default();

        // Only ask to save if running in interactive mode (not print, not one-shot)
        if !args.print && !args.one_shot {
            use dialoguer::Confirm;
            let should_save = Confirm::new()
                .with_prompt("Save this location for future use?")
                .default(false)
                .interact()
                .unwrap_or(false);

            if should_save {
                config.set_location(loc, LocationSource::Manual, None);
                config.save().ok(); // Ignore save errors
                info!("Location saved to configuration file");
            } else {
                debug!("Location will not be saved (session only)");
            }
        }

        return Ok((loc, config));
    }

    // Load or create config
    let mut config = Config::load().unwrap_or_default();

    // Priority 2: INI config file manual location
    if let Some(ini_loc) = ini_config.get_manual_location() {
        info!("Using location from INI config: {:.4}, {:.4}", ini_loc.lat, ini_loc.lon);
        return Ok((ini_loc, config));
    }

    // Priority 3: Try GeoClue2 if it's time for daily check
    if config.should_check_geoclue() {
        info!("Checking for automatic location via GeoClue2...");

        if let Ok(loc) = try_geoclue2() {
            info!("Got location from GeoClue2: {:.4}, {:.4}", loc.lat, loc.lon);

            config.set_location(loc, LocationSource::GeoClue2, None);
            config.update_geoclue_check();
            config.save().ok();

            return Ok((loc, config));
        }

        // Mark that we checked, even though it failed
        config.update_geoclue_check();
        config.save().ok();
    }

    // Priority 4: Use saved TOML configuration
    if let Some(saved_loc) = config.get_location() {
        let source_name = config.location.as_ref().map(|l| match l.source {
            LocationSource::Manual => "manual entry",
            LocationSource::Interactive => "interactive selection",
            LocationSource::GeoClue2 => "GeoClue2",
        }).unwrap_or("unknown");

        if let Some(ref city) = config.location.as_ref().and_then(|l| l.city_name.as_ref()) {
            info!("Using saved location for {}: {:.4}, {:.4} (from {})",
                city, saved_loc.lat, saved_loc.lon, source_name);
        } else {
            info!("Using saved location: {:.4}, {:.4} (from {})",
                saved_loc.lat, saved_loc.lon, source_name);
        }

        return Ok((saved_loc, config));
    }

    // Priority 5: Interactive selection
    if args.no_auto_location {
        eprintln!("Error: --no-auto-location requires -l LAT:LON or saved configuration");
        std::process::exit(1);
    }

    eprintln!("\nNo location configured and automatic detection unavailable.");
    let loc = interactive::select_location_interactive()?;

    // Save for future use
    let city_name = format!("Selected city"); // Could be improved
    config.set_location(loc, LocationSource::Interactive, Some(city_name));
    config.save().ok();

    Ok((loc, config))
}

/// Try to get location from GeoClue2
fn try_geoclue2() -> Result<Location, String> {
    let mut provider = GeoClue2LocationProvider::new();
    provider.init()?;
    provider.start()?;

    // Wait for location
    debug!("Waiting for location from GeoClue2...");
    std::thread::sleep(Duration::from_secs(5));

    provider.get_location()
}

/// Build transition scheme from args and INI config
fn build_transition_scheme(
    args: &Args,
    ini_config: &config_ini::RedshiftConfig,
) -> Result<TransitionScheme, String> {
    let mut scheme = TransitionScheme::default();

    /* Set temperatures from merged args */
    scheme.day.temperature = args.temp_day;
    scheme.night.temperature = args.temp_night;

    /* Parse and apply brightness from CLI or INI */
    if let Some(ref brightness_str) = args.brightness {
        let (day, night) = config_ini::parse_brightness_string(brightness_str)?;
        scheme.day.brightness = day;
        scheme.night.brightness = night;
    } else {
        if let Some(day) = ini_config.brightness_day {
            scheme.day.brightness = day;
        }
        if let Some(night) = ini_config.brightness_night {
            scheme.night.brightness = night;
        }
    }

    /* Parse and apply gamma from CLI or INI */
    if let Some(ref gamma_str) = args.gamma {
        let gamma = config_ini::parse_gamma_string(gamma_str)?;
        scheme.day.gamma = gamma;
        scheme.night.gamma = gamma;
    } else {
        if let Some(gamma) = ini_config.gamma_day {
            scheme.day.gamma = gamma;
        }
        if let Some(gamma) = ini_config.gamma_night {
            scheme.night.gamma = gamma;
        }
    }

    /* Apply elevation settings from INI */
    if let Some(high) = ini_config.elevation_high {
        scheme.high = high;
    }
    if let Some(low) = ini_config.elevation_low {
        scheme.low = low;
    }

    /* Apply time-based transition if specified */
    if let Some(dawn) = ini_config.dawn_time {
        scheme.use_time = true;
        scheme.dawn = dawn;
    }
    if let Some(dusk) = ini_config.dusk_time {
        scheme.use_time = true;
        scheme.dusk = dusk;
    }

    /* Validate brightness bounds */
    if scheme.day.brightness < MIN_BRIGHTNESS || scheme.day.brightness > MAX_BRIGHTNESS {
        return Err(format!(
            "Day brightness must be between {} and {}",
            MIN_BRIGHTNESS, MAX_BRIGHTNESS
        ));
    }
    if scheme.night.brightness < MIN_BRIGHTNESS || scheme.night.brightness > MAX_BRIGHTNESS {
        return Err(format!(
            "Night brightness must be between {} and {}",
            MIN_BRIGHTNESS, MAX_BRIGHTNESS
        ));
    }

    /* Validate gamma bounds */
    for &gamma in &scheme.day.gamma {
        if gamma < MIN_GAMMA || gamma > MAX_GAMMA {
            return Err(format!(
                "Day gamma must be between {} and {}",
                MIN_GAMMA, MAX_GAMMA
            ));
        }
    }
    for &gamma in &scheme.night.gamma {
        if gamma < MIN_GAMMA || gamma > MAX_GAMMA {
            return Err(format!(
                "Night gamma must be between {} and {}",
                MIN_GAMMA, MAX_GAMMA
            ));
        }
    }

    Ok(scheme)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Args::parse();

    /* Initialize logger based on verbosity level */
    let log_level = match args.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .format_timestamp(if args.verbose >= 2 {
            Some(env_logger::fmt::TimestampPrecision::Millis)
        } else {
            Some(env_logger::fmt::TimestampPrecision::Seconds)
        })
        .init();

    debug!("Logger initialized at level: {:?}", log_level);

    /* Install signal handlers for graceful shutdown and mode toggling */
    signals::install_handlers()?;

    /* Load INI configuration file */
    let ini_config = config_ini::RedshiftConfig::load().unwrap_or_default();

    /* Merge INI config with CLI args (CLI takes priority) */
    args.merge_with_ini(&ini_config);

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

    /* Determine location using priority system:
       1. Command-line argument (-l LAT:LON)
       2. INI config file manual location
       3. Saved TOML configuration file
       4. GeoClue2 automatic detection (with daily retry)
       5. Interactive selection (country/city list)
    */
    let (location, mut config) = determine_location_with_ini(&args, &ini_config)?;

    /* Set up gamma method */
    let mut gamma_method: Box<dyn GammaMethod> = match args.method {
        GammaMethodChoice::Randr => Box::new(RandrGammaMethod::new()),
        GammaMethodChoice::Dummy => Box::new(DummyGammaMethod::new()),
    };

    info!("Initializing gamma method: {}", gamma_method.name());
    gamma_method.init()?;
    gamma_method.start()?;

    /* Create transition scheme from args and INI config */
    let scheme = build_transition_scheme(&args, &ini_config)?;

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

    /* Create gamma restore guard to ensure cleanup on exit or panic */
    let mut gamma_guard = GammaRestoreGuard::new(gamma_method.as_mut());

    /* Apply color temperature */
    info!("Period: {}", period.name());
    debug!(
        "Color temperature: {}K, Brightness: {:.2}, Gamma: {:.2}/{:.2}/{:.2}",
        color_setting.temperature,
        color_setting.brightness,
        color_setting.gamma[0],
        color_setting.gamma[1],
        color_setting.gamma[2]
    );

    gamma_guard.get_mut().set_temperature(&color_setting, false)?;

    if args.one_shot {
        /* For one-shot mode, don't restore gamma on exit */
        gamma_guard.disable_restore();
        return Ok(());
    }

    /* Continual mode - continuously adjust color temperature */
    run_continual_mode(&location, &scheme, &mut gamma_guard)?;

    Ok(())
}

/* Run continual mode loop.
   This is the main loop of the continual mode which keeps track of the
   current time and continuously updates the screen to the appropriate
   color temperature. Also handles signals for toggling and clean exit. */
fn run_continual_mode(
    location: &Location,
    scheme: &TransitionScheme,
    gamma_guard: &mut GammaRestoreGuard,
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

    /* State for signal handling */
    let mut disabled = false;
    let mut prev_disabled = true; /* Start as true to trigger initial status print */
    let mut done = false; /* Set to true when starting shutdown fade */

    debug!("Starting continual mode loop");
    debug!("Initial color temperature: {}K, Brightness: {:.2}", interp.temperature, interp.brightness);

    /* Continuously adjust color temperature */
    loop {
        /* Check for toggle signal (SIGUSR1) */
        if signals::check_toggle() && !done {
            disabled = !disabled;
            info!("Status: {}", if disabled { "Disabled" } else { "Enabled" });
        }

        /* Check for exit signal (SIGINT/SIGTERM) */
        if signals::is_exiting() {
            if done {
                /* Second signal during fade - stop immediately */
                debug!("Second exit signal received, stopping immediately");
                break;
            } else {
                /* First signal - start shutdown fade */
                info!("Exit signal received, starting shutdown fade");
                done = true;
                disabled = true;
                signals::clear_exiting();
            }
        }

        /* Print status change */
        if disabled != prev_disabled {
            info!("Status: {}", if disabled { "Disabled" } else { "Enabled" });
        }
        prev_disabled = disabled;

        /* When disabled, use neutral temperature; otherwise calculate from solar position */
        let mut target_interp = if disabled {
            /* Neutral temperature (6500K) when disabled */
            ColorSetting {
                temperature: 6500,
                brightness: 1.0,
                gamma: [1.0, 1.0, 1.0],
            }
        } else {
            /* Get current time */
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            /* Current angular elevation of the sun */
            let elevation = solar::solar_elevation(now, location.lat as f64, location.lon as f64);
            trace!("Solar elevation: {:.2}°", elevation);

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
            let mut temp_interp = ColorSetting::default();
            interpolate_transition_scheme(scheme, transition_prog, &mut temp_interp);

            /* Print period if it changed during this update,
               or if we are in the transition period. In transition we
               print the progress, so we always print it in that case. */
            if period != prev_period || period == Period::Transition {
                match period {
                    Period::Transition => {
                        info!("Period: Transition ({:.1}%)", transition_prog * 100.0);
                        debug!("Transition progress: {:.3} (elevation: {:.2}°)", transition_prog, elevation);
                    }
                    _ => {
                        info!("Period: {}", period.name());
                    }
                }
            }
            prev_period = period;

            temp_interp
        };

        /* Start fade if the parameter differences are too big to apply instantly. */
        if (fade_length == 0 && color_setting_diff_is_major(&interp, &target_interp))
            || (fade_length != 0 && color_setting_diff_is_major(&target_interp, &prev_target_interp))
        {
            debug!("Starting fade: {} steps", FADE_LENGTH);
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
            trace!("Fade progress: {}/{} (alpha: {:.3})", fade_time, fade_length, alpha);

            if fade_time > fade_length {
                debug!("Fade complete");
                fade_time = 0;
                fade_length = 0;
            }
        } else {
            interp = target_interp;
        }

        if prev_target_interp.temperature != target_interp.temperature {
            info!("Color temperature: {}K", target_interp.temperature);
        }
        if prev_target_interp.brightness != target_interp.brightness {
            debug!("Brightness: {:.2}", target_interp.brightness);
        }

        /* Adjust temperature */
        gamma_guard.get_mut().set_temperature(&interp, false)?;

        /* Save target color setting as previous */
        prev_target_interp = target_interp;

        /* If shutdown was requested and fade is complete, exit */
        if done && fade_length == 0 {
            break;
        }

        /* Sleep length depends on whether a fade is ongoing. */
        let delay = if fade_length != 0 {
            SLEEP_DURATION_SHORT
        } else {
            SLEEP_DURATION
        };

        std::thread::sleep(Duration::from_millis(delay));
    }

    Ok(())
}
