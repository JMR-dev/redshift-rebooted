/* gamma_guard.rs -- Gamma restoration guard for cleanup
 * This module provides a RAII guard that ensures gamma is restored
 * even if the program crashes or panics.
 */

use crate::gamma::GammaMethod;
use crate::types::ColorSetting;

/* Guard that restores gamma to neutral (6500K) on drop.
 * This ensures cleanup happens on normal exit, panic, or signal. */
pub struct GammaRestoreGuard<'a> {
    gamma_method: &'a mut dyn GammaMethod,
    restore_on_drop: bool,
}

impl<'a> GammaRestoreGuard<'a> {
    /* Create a new gamma restore guard.
     * The gamma will be restored when this guard is dropped. */
    pub fn new(gamma_method: &'a mut dyn GammaMethod) -> Self {
        GammaRestoreGuard {
            gamma_method,
            restore_on_drop: true,
        }
    }

    /* Disable automatic restoration.
     * Call this if you want to keep the current gamma on exit. */
    #[allow(dead_code)]
    pub fn disable_restore(&mut self) {
        self.restore_on_drop = false;
    }

    /* Get mutable reference to the gamma method.
     * This allows using the gamma method while the guard is active. */
    pub fn get_mut(&mut self) -> &mut dyn GammaMethod {
        self.gamma_method
    }
}

impl<'a> Drop for GammaRestoreGuard<'a> {
    fn drop(&mut self) {
        if self.restore_on_drop {
            /* Restore to neutral temperature (6500K) */
            let neutral = ColorSetting {
                temperature: 6500,
                brightness: 1.0,
                gamma: [1.0, 1.0, 1.0],
            };

            /* Ignore errors during cleanup - we're likely shutting down anyway */
            let _ = self.gamma_method.set_temperature(&neutral, false);
        }
    }
}
