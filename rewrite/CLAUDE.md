# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust rewrite of Redshift, a screen color temperature adjustment tool. The goal is to modernize the codebase while maintaining feature parity with the original C implementation located in `../legacy/`.

The original Redshift project:

- Adjusts screen color temperature based on time of day using gamma ramps
- Consists of a C daemon (`redshift`) and Python GUI (`redshift-gtk`)
- Uses GNU Autotools build system
- See `../CLAUDE.md` for detailed documentation of the original architecture

## Build System

This rewrite uses Cargo, Rust's standard build tool.

### Common Commands

```shell
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the binary
cargo run

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Project Status

**Current state:** Initial project setup with minimal "Hello, world!" implementation.

The rewrite is in early stages. The original C codebase in `../legacy/` should be referenced for:

- Architecture patterns (adjustment methods, location providers, solar calculations)
- Platform-specific implementations
- Configuration file format
- Command-line interface design

## Architecture Goals

The Rust rewrite should maintain the modular architecture of the original:

**Core modules to implement:**

- Gamma adjustment backends (DRM, RANDR, VidMode)
- Location providers (manual, GeoClue2)
- Solar position calculations
- Color temperature to RGB conversion
- Configuration parsing
- Main event loop and signal handling

**Key architectural decisions from original:**

- Adjustment methods are pluggable (different display systems)
- Location providers are queried at startup only
- Main loop sets gamma every few seconds/minutes
- Short transitions (~10s) at startup, long transitions (~50min) otherwise
- Signal handling: SIGUSR1 toggles day/night, SIGINT/SIGTERM restores gamma

## Reference Implementation

The legacy C implementation is located in `../legacy/src/`:

**Entry point:** `redshift.c` - Main loop, signal handling, transition logic
**Adjustment:** `gamma-*.c/h` - Platform-specific gamma ramp manipulation
**Location:** `location-*.c/h` - Geographic coordinate providers
**Solar:** `solar.c/h` - Day/night calculations based on sun position
**Color:** `colorramp.c/h` - Temperature to RGB gamma conversion
**Config:** `config-ini.c/h`, `options.c/h` - Configuration and CLI parsing

## Dependencies

Currently the project has no dependencies. As features are implemented, consider:

**Potential crates:**

- `clap` or `structopt` for CLI argument parsing
- `serde` and `toml`/`ini` for configuration file parsing
- `chrono` for time calculations
- Platform-specific crates for display/gamma control
- `getopts` if maintaining closer compatibility with original

## Coding Style

Follow standard Rust conventions:

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Prefer idiomatic Rust patterns over direct C translations
- Document public APIs with doc comments (`///`)

## Platform Support

This rewrite targets Linux only:

- **DRM:** Direct Rendering Manager (TTY/framebuffer)
- **RANDR:** X11 RandR extension (multi-output support, preferred)
- **VidMode:** X11 VidMode extension (legacy, single output)

Each method requires different system APIs for gamma manipulation. Conditional compilation via `cfg` attributes will be needed for feature detection.

## Configuration

Configuration files should be in INI format at:

- `~/.config/redshift.conf` (XDG standard location)

Maintain this format for compatibility with the original.

## Testing Strategy

The original includes a dummy adjustment method for testing without actual display changes. Implement similar testing infrastructure:

- Unit tests for solar calculations
- Unit tests for color temperature conversion
- Integration tests with dummy/mock adjustment methods
- Always generate a coverage report at the end of a task and write tests for any uncovered lines or branches that were not previously ignored or skipped.
