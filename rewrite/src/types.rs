/// Core types for Redshift
/// Ported from legacy/src/redshift.h

/// The color temperature when no adjustment is applied
pub const NEUTRAL_TEMP: i32 = 6500;

/// Bounds for parameters
pub const MIN_LAT: f32 = -90.0;
pub const MAX_LAT: f32 = 90.0;
pub const MIN_LON: f32 = -180.0;
pub const MAX_LON: f32 = 180.0;
pub const MIN_TEMP: i32 = 1000;
pub const MAX_TEMP: i32 = 25000;
pub const MIN_BRIGHTNESS: f32 = 0.1;
pub const MAX_BRIGHTNESS: f32 = 1.0;
pub const MIN_GAMMA: f32 = 0.1;
pub const MAX_GAMMA: f32 = 10.0;

/// Geographic location
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub lat: f32,
    pub lon: f32,
}

/// Periods of day
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    None,
    Daytime,
    Night,
    Transition,
}

impl Period {
    pub fn name(&self) -> &'static str {
        match self {
            Period::None => "None",
            Period::Daytime => "Daytime",
            Period::Night => "Night",
            Period::Transition => "Transition",
        }
    }
}

/// Color setting with temperature, gamma, and brightness
#[derive(Debug, Clone, Copy)]
pub struct ColorSetting {
    pub temperature: i32,
    pub gamma: [f32; 3],
    pub brightness: f32,
}

impl Default for ColorSetting {
    fn default() -> Self {
        Self {
            temperature: NEUTRAL_TEMP,
            gamma: [1.0, 1.0, 1.0],
            brightness: 1.0,
        }
    }
}

/// Program operation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramMode {
    Continual,
    OneShot,
    Print,
    Reset,
    Manual,
}

/// Time range in seconds from midnight
#[derive(Debug, Clone, Copy)]
pub struct TimeRange {
    pub start: i32,
    pub end: i32,
}

/// Transition scheme defining solar elevations and color settings
#[derive(Debug, Clone, Copy)]
pub struct TransitionScheme {
    pub high: f64,
    pub low: f64,
    pub use_time: bool,
    pub dawn: TimeRange,
    pub dusk: TimeRange,
    pub day: ColorSetting,
    pub night: ColorSetting,
}

impl Default for TransitionScheme {
    fn default() -> Self {
        Self {
            high: 3.0,
            low: -6.0,
            use_time: false,
            dawn: TimeRange { start: 0, end: 0 },
            dusk: TimeRange { start: 0, end: 0 },
            day: ColorSetting::default(),
            night: ColorSetting {
                temperature: 3500,
                gamma: [1.0, 1.0, 1.0],
                brightness: 1.0,
            },
        }
    }
}
