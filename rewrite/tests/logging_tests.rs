/// Tests for logging functionality and verbosity levels

use std::process::Command;

#[test]
fn test_no_verbose_flag_shows_minimal_output() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // With no verbose flag, should not see DEBUG, INFO, or TRACE logs
    assert!(!stderr.contains("DEBUG"), "No verbose flag should not show DEBUG logs");
    assert!(!stderr.contains("INFO"), "No verbose flag should not show INFO logs");
    assert!(!stderr.contains("TRACE"), "No verbose flag should not show TRACE logs");
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
    assert!(stderr.contains("CRTC"), "Should log CRTC details");

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
    assert!(stderr.contains("saved") && stderr.contains("gamma ramp values"), "Should log gamma ramp details");
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
    assert!(stderr.contains("Connected to X server"), "Should log X connection");
    assert!(stderr.contains("Found") && stderr.contains("CRTCs"), "Should log CRTC count");
    assert!(stderr.contains("Successfully initialized") && stderr.contains("CRTCs"), "Should log success");
}

#[test]
fn test_debug_logging_shows_randr_version() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("RandR version:"), "Should log RandR version at debug level");
    assert!(stderr.contains("Getting screen resources"), "Should log screen resource gathering");
}

#[test]
fn test_trace_logging_shows_gamma_ramp_details() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-l", "40:-74", "-p", "-vvv"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("saved") && stderr.contains("gamma ramp values"),
        "Should log saved gamma ramp count at trace level");
    // Note: "Setting temperature for CRTC" only appears in continual mode, not print mode
    // In print mode, we can check for CRTC details
    assert!(stderr.contains("CRTC") && stderr.contains("saved"),
        "Should log CRTC details at trace level");
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

    // Should log the period (Night, Daytime, or Transition)
    assert!(combined.contains("Period:"), "Should log current period");
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

    // At debug level, should see color settings in DEBUG logs or output
    assert!(combined.contains("Color temperature:"),
        "Should show color temperature");
    assert!(combined.contains("Brightness:"), "Should show brightness");
    assert!(combined.contains("Gamma:"), "Should show gamma values");
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
            // Look for pattern like .NNN]
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
