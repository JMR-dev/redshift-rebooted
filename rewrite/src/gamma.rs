/// Gamma adjustment methods
/// Ported from legacy/src/gamma-*.c

use crate::types::ColorSetting;
use std::fmt;

/// Trait for gamma adjustment methods
pub trait GammaMethod {
    /// Initialize the method with optional configuration
    fn init(&mut self) -> Result<(), String>;

    /// Start the method (allocate resources, establish connections)
    fn start(&mut self) -> Result<(), String>;

    /// Set a color temperature adjustment
    fn set_temperature(&mut self, setting: &ColorSetting, preserve: bool)
        -> Result<(), String>;

    /// Restore the display to original state
    fn restore(&mut self);

    /// Get the method name
    fn name(&self) -> &str;

    /// Print help information
    fn print_help(&self);
}

/// Dummy gamma method (no-op, for testing)
/// Ported from legacy/src/gamma-dummy.c
pub struct DummyGammaMethod {}

impl DummyGammaMethod {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DummyGammaMethod {
    fn default() -> Self {
        Self::new()
    }
}

impl GammaMethod for DummyGammaMethod {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        eprintln!(
            "WARNING: Using dummy gamma method! Display will not be affected by this gamma method."
        );
        Ok(())
    }

    fn set_temperature(
        &mut self,
        setting: &ColorSetting,
        _preserve: bool,
    ) -> Result<(), String> {
        println!("Temperature: {}", setting.temperature);
        Ok(())
    }

    fn restore(&mut self) {
        // No-op
    }

    fn name(&self) -> &str {
        "dummy"
    }

    fn print_help(&self) {
        println!("Does not affect the display but prints the color temperature to the terminal.");
        println!();
    }
}

impl fmt::Display for DummyGammaMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dummy")
    }
}
