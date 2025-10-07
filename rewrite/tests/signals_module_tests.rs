/* Unit tests for signals module */

use redshift_rebooted::signals;
use serial_test::serial;

/* Install handlers once for all tests */
#[ctor::ctor]
fn init() {
    let _ = signals::install_handlers();
}

#[test]
fn test_install_handlers() {
    /* Should successfully install signal handlers */
    let result = signals::install_handlers();
    assert!(result.is_ok(), "Should install handlers without error (can be called multiple times)");
}

#[test]
#[serial(signals)]
fn test_is_exiting_initial_state() {
    /* Clear state first */
    signals::clear_exiting();

    /* Should not be exiting */
    assert!(!signals::is_exiting(), "Should not be exiting after clear");
}

#[test]
#[serial(signals)]
fn test_check_toggle_initial_state() {
    /* Clear state first */
    signals::check_toggle();

    /* Should not have toggle requested after clearing */
    assert!(!signals::check_toggle(), "Should not have toggle requested after clearing");
}

#[test]
#[serial(signals)]
fn test_check_toggle_clears_flag() {
    /* check_toggle should return true only once per signal */
    /* Note: This test can't easily set the flag without sending actual signals,
       so this is more of a behavior documentation test */

    /* Clear state first */
    signals::check_toggle();

    /* First check - should be false (no signal sent) */
    let first = signals::check_toggle();
    /* Second check - should still be false */
    let second = signals::check_toggle();

    assert_eq!(first, second, "Multiple checks without signal should return same value");
}

#[test]
#[serial(signals)]
fn test_clear_exiting() {
    /* clear_exiting should reset the exiting flag */
    signals::clear_exiting();

    /* After clearing, should not be exiting */
    assert!(!signals::is_exiting(), "Should not be exiting after clear");
}

#[test]
fn test_signal_handlers_thread_safe() {
    /* Signal handlers use Arc<AtomicBool> which is thread-safe */
    use std::thread;

    signals::install_handlers().expect("Failed to install handlers");

    /* Spawn multiple threads checking signals */
    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..100 {
                    let _ = signals::is_exiting();
                    let _ = signals::check_toggle();
                }
            })
        })
        .collect();

    /* All threads should complete without issue */
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_multiple_install_handlers() {
    /* Installing handlers multiple times should work */
    for _ in 0..3 {
        let result = signals::install_handlers();
        assert!(result.is_ok(), "Multiple installs should succeed");
    }
}

#[cfg(unix)]
#[test]
#[serial(signals)]  /* Use a specific key to ensure proper serialization */
fn test_actual_sigusr1_signal() {
    use std::thread;
    use std::time::Duration;

    /* This test is potentially flaky due to signal delivery timing.
     * We retry a few times to reduce false failures. */
    let mut success = false;

    for attempt in 0..3 {
        /* Clear any previous state */
        signals::clear_toggle();

        /* Send SIGUSR1 to self */
        unsafe {
            libc::kill(std::process::id() as i32, libc::SIGUSR1);
        }

        /* Poll for the signal with timeout - peek without clearing */
        let mut detected = false;
        for _ in 0..30 {  /* Try for up to 300ms */
            thread::sleep(Duration::from_millis(10));
            if signals::is_toggle_requested() {
                detected = true;
                break;
            }
        }

        if detected {
            /* Clear and verify */
            assert!(signals::check_toggle(), "Should still be set");
            assert!(!signals::check_toggle(), "Toggle flag should be cleared after check");
            success = true;
            break;
        }

        if attempt < 2 {
            eprintln!("Signal delivery attempt {} failed, retrying...", attempt + 1);
            thread::sleep(Duration::from_millis(50));
        }
    }

    assert!(success, "Should detect SIGUSR1 within 3 attempts");
}

#[cfg(unix)]
#[test]
#[serial(signals)]
fn test_actual_sigint_signal() {
    use std::thread;
    use std::time::Duration;


    /* Clear any previous state */
    signals::clear_exiting();

    /* Send SIGINT to self */
    unsafe {
        libc::kill(std::process::id() as i32, libc::SIGINT);
    }

    /* Give signal time to be processed */
    thread::sleep(Duration::from_millis(100));

    /* Should detect exit signal */
    assert!(signals::is_exiting(), "Should detect SIGINT");
}

#[cfg(unix)]
#[test]
#[serial(signals)]
fn test_actual_sigterm_signal() {
    use std::thread;
    use std::time::Duration;


    /* Clear any previous state */
    signals::clear_exiting();

    /* Send SIGTERM to self */
    unsafe {
        libc::kill(std::process::id() as i32, libc::SIGTERM);
    }

    /* Give signal time to be processed */
    thread::sleep(Duration::from_millis(100));

    /* Should detect exit signal */
    assert!(signals::is_exiting(), "Should detect SIGTERM");
}

#[cfg(unix)]
#[test]
#[serial(signals)]
fn test_sigint_and_sigterm_both_set_exiting() {
    use std::thread;
    use std::time::Duration;


    /* Test SIGINT */
    signals::clear_exiting();
    unsafe {
        libc::kill(std::process::id() as i32, libc::SIGINT);
    }
    thread::sleep(Duration::from_millis(100));
    assert!(signals::is_exiting(), "SIGINT should set exiting");

    /* Test SIGTERM */
    signals::clear_exiting();
    unsafe {
        libc::kill(std::process::id() as i32, libc::SIGTERM);
    }
    thread::sleep(Duration::from_millis(100));
    assert!(signals::is_exiting(), "SIGTERM should set exiting");
}

#[cfg(unix)]
#[test]
#[serial(signals)]
fn test_multiple_sigusr1_signals() {
    use std::thread;
    use std::time::Duration;


    /* Clear state */
    signals::check_toggle();

    /* Send multiple SIGUSR1 signals */
    for _ in 0..3 {
        unsafe {
            libc::kill(std::process::id() as i32, libc::SIGUSR1);
        }
        thread::sleep(Duration::from_millis(10));
    }

    thread::sleep(Duration::from_millis(100));

    /* Should detect at least one toggle (flag might be set/cleared multiple times) */
    let detected = signals::check_toggle();
    /* The behavior here is that the flag is set to true, so even with multiple signals,
       check_toggle will return true once, then false */
    assert!(detected, "Should detect toggle from multiple SIGUSR1 signals");
}
