# Signal Handling Test Suite

This document describes the comprehensive test suite for signal handling in Redshift.

## Test Files

### 1. Unit Tests: `tests/signals_module_tests.rs`

Tests for the `signals` module itself:

- **`test_install_handlers`** - Verifies signal handlers can be installed without error
- **`test_is_exiting_initial_state`** - Checks initial state of exiting flag
- **`test_check_toggle_initial_state`** - Checks initial state of toggle flag
- **`test_check_toggle_clears_flag`** - Verifies toggle flag is cleared after check
- **`test_clear_exiting`** - Tests clearing the exiting flag
- **`test_signal_handlers_thread_safe`** - Verifies thread safety of signal handlers
- **`test_multiple_install_handlers`** - Tests installing handlers multiple times
- **`test_actual_sigusr1_signal`** - Sends real SIGUSR1 and verifies detection
- **`test_actual_sigint_signal`** - Sends real SIGINT and verifies detection
- **`test_actual_sigterm_signal`** - Sends real SIGTERM and verifies detection
- **`test_sigint_and_sigterm_both_set_exiting`** - Verifies both signals set the same flag
- **`test_multiple_sigusr1_signals`** - Tests handling multiple toggle signals

**Result:** ✅ All 12 tests passing

### 2. Unit Tests: `tests/gamma_guard_tests.rs`

Tests for the `GammaRestoreGuard` RAII guard:

- **`test_gamma_guard_restores_on_drop`** - Verifies gamma restoration on normal drop
- **`test_gamma_guard_can_be_disabled`** - Tests disabling automatic restoration
- **`test_gamma_guard_get_mut`** - Tests mutable access to gamma method
- **`test_gamma_guard_restores_on_panic`** - Verifies gamma restoration even on panic
- **`test_multiple_guards_sequential`** - Tests using multiple guards sequentially
- **`test_guard_restores_neutral_values`** - Verifies restoration to neutral (6500K)

**Result:** ✅ All 6 tests passing

### 3. Integration Tests: `tests/signal_tests.rs`

End-to-end tests using actual redshift processes:

- **`test_sigusr1_toggle`** - Toggle disabled/enabled with SIGUSR1, verify gamma restore
- **`test_sigusr1_double_toggle`** - Toggle off and back on, verify state changes
- **`test_sigterm_clean_shutdown`** - Clean shutdown with SIGTERM, verify gamma restore
- **`test_sigint_clean_shutdown`** - Clean shutdown with SIGINT (Ctrl+C)
- **`test_double_sigterm_immediate_exit`** - [IGNORED] Second SIGTERM causes immediate exit
- **`test_sigusr1_during_shutdown_ignored`** - Toggle during shutdown is ignored
- **`test_one_shot_mode_no_signals`** - One-shot mode exits without signals
- **`test_print_mode_no_signals`** - Print mode exits without signals
- **`test_gamma_restoration_fade`** - Verifies smooth fade to neutral during shutdown

**Result:** ✅ 8 tests passing, 1 ignored (flaky due to timing)

## Test Coverage

### Signal Handling Features Tested

✅ **SIGUSR1 (Toggle)**
- Toggles between enabled/disabled states
- Restores gamma to 6500K when disabled
- Can toggle multiple times
- Ignored during shutdown

✅ **SIGINT/SIGTERM (Shutdown)**
- First signal starts shutdown with gamma restoration fade
- Gamma fades to neutral 6500K
- Clean exit with status code 0
- Second signal causes immediate exit (verified manually)

✅ **Gamma Restoration**
- RAII guard ensures gamma restore on all exit paths
- Restores even on panic
- Neutral values: 6500K, brightness 1.0, gamma [1.0, 1.0, 1.0]
- Smooth fade during shutdown

✅ **Thread Safety**
- Signal handlers use Arc<AtomicBool> for thread-safe access
- Multiple threads can check signals concurrently
- No race conditions

## Running the Tests

**Important:** Integration tests require the binary to be built first:
```bash
cargo build
```

### Run all tests:
```bash
# Build first, then run tests
cargo build && cargo test
```

### Run specific test suites:
```bash
# Signals module unit tests
cargo test --test signals_module_tests

# Gamma guard unit tests
cargo test --test gamma_guard_tests

# Integration tests (requires pre-built binary)
cargo build && cargo test --test signal_tests
```

### Run with output:
```bash
cargo test --test signal_tests -- --nocapture
```

### Run ignored tests:
```bash
cargo test -- --ignored
```

## Manual Testing

The double SIGTERM behavior (immediate exit on second signal during fade) is best tested manually:

```bash
# Terminal 1
cargo run -- -l 40:-74 -m dummy -v

# Terminal 2
pkill -TERM -f redshift-rebooted
sleep 0.1
pkill -TERM -f redshift-rebooted
```

Expected: Process exits immediately without completing the fade.

## Notes

- **Integration tests** use the pre-built binary directly to avoid parallel build conflicts
- **Signal module unit tests** use `#[serial]` attribute to run sequentially (global state)
  - The signal handlers use global atomic flags that are shared across tests
  - The `serial_test` crate ensures tests that interact with signals run one at a time
- Tests can run in parallel at the test suite level (different test files)
- Some tests are timing-sensitive and may occasionally be flaky on slow systems
- The `wait-timeout` crate is used for reliable process timeout handling in integration tests
- Tests use `libc::kill()` to send signals to child processes
- Always run `cargo build` before running integration tests
