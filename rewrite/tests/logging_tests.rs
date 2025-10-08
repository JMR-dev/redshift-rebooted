/// Tests for logging functionality and verbosity levels
///
/// Note: Tests that require X11/RandR will check if DISPLAY is set and skip
/// or adjust expectations accordingly for CI environments.

use std::process::Command;

/// Helper to check if X11 is available (needed for RandR tests)
fn is_x11_available() -> bool {
    std::env::var("DISPLAY").is_ok()
}

#[test]
fn test_no_verbose_flag_shows_minimal_output() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // With no verbose flag, should not see DEBUG, INFO, or TRACE logs
    // (unless there's an error, which is fine)
    if output.status.success() {
        assert!(!stderr.contains("DEBUG"), "No verbose flag should not show DEBUG logs");
        assert!(!stderr.contains("INFO"), "No verbose flag should not show INFO logs");
        assert!(!stderr.contains("TRACE"), "No verbose flag should not show TRACE logs");
    }
}

#[test]
fn test_single_v_shows_info_logs() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-v"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // With -v, should see INFO logs
    assert!(stderr.contains("INFO"), "Single -v should show INFO logs");
    assert!(stderr.contains("Using location from command-line"), "Should log location source");
    assert!(stderr.contains("Initializing gamma method"), "Should log gamma initialization");

    // Should not see DEBUG or TRACE
    assert!(!stderr.contains("DEBUG"), "Single -v should not show DEBUG logs");
    assert!(!stderr.contains("TRACE"), "Single -v should not show TRACE logs");
}

#[test]
fn test_double_v_shows_debug_logs() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // With -vv, should see both INFO and DEBUG logs
    assert!(stderr.contains("INFO"), "Double -v should show INFO logs");
    assert!(stderr.contains("DEBUG"), "Double -v should show DEBUG logs");
    assert!(stderr.contains("Logger initialized at level: Debug"), "Should log logger initialization");

    // CRTC details only appear if X11 is available
    if is_x11_available() && output.status.success() {
        assert!(stderr.contains("CRTC"), "Should log CRTC details when X11 is available");
    }

    // Should not see TRACE
    assert!(!stderr.contains("TRACE"), "Double -v should not show TRACE logs");
}

#[test]
fn test_triple_v_shows_trace_logs() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vvv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // With -vvv, should see INFO, DEBUG, and TRACE logs
    assert!(stderr.contains("INFO"), "Triple -v should show INFO logs");
    assert!(stderr.contains("DEBUG"), "Triple -v should show DEBUG logs");
    assert!(stderr.contains("TRACE"), "Triple -v should show TRACE logs");
    assert!(stderr.contains("Logger initialized at level: Trace"), "Should log trace level");
    assert!(stderr.contains("Searching for INI config"), "Should log config search");

    // Gamma ramp details only appear if X11 is available
    if is_x11_available() && output.status.success() {
        assert!(stderr.contains("saved") && stderr.contains("gamma ramp values"),
            "Should log gamma ramp details when X11 is available");
    }
}

#[test]
fn test_location_logging_from_cli() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "48.8566:2.3522", "-p", "-v"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Using location from command-line"), "Should log location source");
    assert!(stderr.contains("48.8566"), "Should log latitude");
    assert!(stderr.contains("2.3522"), "Should log longitude");
}

#[test]
fn test_gamma_initialization_logging() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-v"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Initializing gamma method: randr"), "Should log gamma method");

    // X11-specific logs only appear when X11 is available
    if is_x11_available() && output.status.success() {
        assert!(stderr.contains("Connected to X server") || stderr.contains("Found") && stderr.contains("CRTCs"),
            "Should log X connection or CRTC count when X11 is available");
    }
}

#[test]
fn test_debug_logging_shows_randr_version() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Only check RandR-specific logs if X11 is available
    if is_x11_available() && output.status.success() {
        assert!(stderr.contains("RandR version:") || stderr.contains("Getting screen resources"),
            "Should log RandR details at debug level when X11 is available");
    } else {
        // When X11 is not available, just check that debug logging is working
        assert!(stderr.contains("DEBUG"), "Should have debug logs");
    }
}

#[test]
fn test_trace_logging_shows_gamma_ramp_details() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vvv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Only check gamma-specific logs if X11 is available
    if is_x11_available() && output.status.success() {
        assert!(stderr.contains("saved") && stderr.contains("gamma ramp values"),
            "Should log saved gamma ramp count at trace level when X11 is available");
    } else {
        // When X11 is not available, just check that trace logging is working
        assert!(stderr.contains("TRACE"), "Should have trace logs");
    }
}

#[test]
fn test_config_search_logging() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Searching for INI configuration file"),
        "Should log config search at debug level");
}

#[test]
fn test_trace_shows_config_search_paths() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vvv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Checking:"), "Should show individual config paths at trace level");
}

#[test]
fn test_period_logging() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-v"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stderr, stdout);

    // Period appears in stdout in print mode
    // Only check if the command succeeded (X11 available)
    if is_x11_available() && output.status.success() {
        assert!(combined.contains("Period:"), "Should log current period when X11 is available");
    }
}

#[test]
fn test_color_temperature_logging_at_debug() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vv"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stderr, stdout);

    // Color temperature appears in stdout in print mode
    // Only check if the command succeeded (X11 available)
    if is_x11_available() && output.status.success() {
        assert!(combined.contains("Color temperature:"),
            "Should show color temperature when X11 is available");
        assert!(combined.contains("Brightness:"), "Should show brightness");
        assert!(combined.contains("Gamma:"), "Should show gamma values");
    }
}

#[test]
fn test_timestamp_precision_at_debug_level() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // At -vv (debug), timestamps should have millisecond precision
    // Format: [2025-10-07T23:35:25.071Z DEBUG ...]
    assert!(stderr.contains("Z DEBUG"), "Debug level should have timestamps");
    // Check for milliseconds (the .XXX part before Z)
    let has_millis = stderr.lines()
        .filter(|line| line.contains("DEBUG"))
        .any(|line| {
            // Look for pattern like .NNN before Z
            line.contains('.') && line.chars()
                .skip_while(|&c| c != '.')
                .skip(1)
                .take(3)
                .all(|c| c.is_ascii_digit())
        });
    assert!(has_millis, "Debug timestamps should include milliseconds");
}

#[test]
fn test_verbosity_count_increments() {
    // Test that -vvvv (more than 3) still works and maps to Trace
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vvvv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should still show Trace level (max level)
    assert!(stderr.contains("Logger initialized at level: Trace"),
        "More than 3 -v flags should still map to Trace");
    assert!(stderr.contains("TRACE"), "Should show TRACE logs");
}

#[test]
fn test_logger_initialization_logging() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Logger initialized at level:"),
        "Should log logger initialization at debug level");
}

#[test]
fn test_location_determination_logging() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Determining location using priority system"),
        "Should log location determination at debug level");
}
