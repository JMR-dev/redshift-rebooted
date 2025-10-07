/* signals.rs -- Signal handling for redshift
 * This file provides signal handlers for graceful shutdown and toggling modes.
 *
 * Signals handled:
 * - SIGUSR1: Toggle between enabled/disabled state (restores gamma when disabled)
 * - SIGINT/SIGTERM: Clean shutdown with gamma restoration
 */

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/* Global atomic flags for signal state.
 * These are safe to access from signal handlers and main thread. */
lazy_static::lazy_static! {
    static ref EXITING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref TOGGLE_REQUESTED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

/* Install signal handlers.
 * Returns an error if signal handler registration fails. */
pub fn install_handlers() -> Result<(), Box<dyn std::error::Error>> {
    use signal_hook::consts::signal::*;
    use signal_hook::flag;

    /* SIGINT and SIGTERM set the exiting flag */
    flag::register(SIGINT, Arc::clone(&EXITING))?;
    flag::register(SIGTERM, Arc::clone(&EXITING))?;

    /* SIGUSR1 sets the toggle flag */
    flag::register(SIGUSR1, Arc::clone(&TOGGLE_REQUESTED))?;

    Ok(())
}

/* Check if an exit signal (SIGINT or SIGTERM) was received.
 * This should be called from the main loop. */
pub fn is_exiting() -> bool {
    EXITING.load(Ordering::Relaxed)
}

/* Check if a toggle signal (SIGUSR1) was received.
 * This returns true only once per signal, then clears the flag. */
pub fn check_toggle() -> bool {
    TOGGLE_REQUESTED.swap(false, Ordering::Relaxed)
}

/* Clear the exiting flag. Used after starting shutdown fade. */
pub fn clear_exiting() {
    EXITING.store(false, Ordering::Relaxed);
}
