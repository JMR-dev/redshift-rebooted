# Redshift Rewrite in Rust

This is a Rust rewrite of Redshift, a screen color temperature adjustment tool. This initial version includes the core functionality needed to calculate and display color temperatures based on geographic location and time of day.

## Current Status

**Phase 1: Core Foundation** ✅ Complete
- Module structure set up with separate files for types, solar, colorramp, gamma, and location
- Core types ported from C: `Location`, `ColorSetting`, `Period`, `TransitionScheme`, `ProgramMode`
- Solar calculations fully ported with astronomical algorithms for day/night timing
- Color ramp logic ported with blackbody color table for temperature-to-RGB conversion

**Phase 2: Basic Functionality** ✅ Complete
- Dummy gamma method implemented (no-op for safe testing)
- Manual location provider implemented (lat/lon specification)
- CLI argument parsing using `clap` crate
- Basic main loop that calculates solar position and applies color temperature

**Phase 3: Basic Testing** ✅ Complete
- Solar calculations tested and working correctly
- Color temperature output tested with dummy method

**Phase 4: Continual Mode** ✅ Complete
- Main event loop implemented with periodic updates
- Smooth fade animations between color temperatures
- Intelligent sleep intervals (5s normal, 100ms during fades)
- Period change detection and verbose status updates

## Building

```bash
cd rewrite
cargo build --release
```

## Usage

The basic command matches the legacy C version:

```bash
# Print current color temperature for a location
./target/debug/redshift-rebooted -l 40.7:-74.0 -m dummy -pv

# Continual mode (continuously updates temperature)
./target/debug/redshift-rebooted -l 40.7:-74.0 -m dummy -v

# One-shot mode (set temperature once and exit)
./target/debug/redshift-rebooted -l 12:-34 -m dummy -o

# Custom day/night temperatures
./target/debug/redshift-rebooted -l 12:-34 -t 5500 --temp-night 3000 -p
```

### Options

- `-l, --location <LAT:LON>` - Location as latitude:longitude (required)
- `-m, --method <METHOD>` - Gamma adjustment method (currently only 'dummy')
- `-o, --one-shot` - Set temperature once and exit
- `-p, --print` - Print current settings and exit
- `-v, --verbose` - Verbose output
- `-t, --temp-day` - Day temperature in Kelvin (default: 6500)
- `--temp-night` - Night temperature in Kelvin (default: 3500)

## Architecture

### Module Structure

```
src/
├── main.rs         - Entry point, CLI parsing, main loop
├── types.rs        - Core type definitions
├── solar.rs        - Solar position calculations
├── colorramp.rs    - Color temperature to RGB conversion
├── gamma.rs        - Gamma adjustment method trait and implementations
└── location.rs     - Location provider trait and implementations
```

### Key Components

**Solar Module** ([solar.rs](src/solar.rs))
- Implements astronomical algorithms from "Astronomical Algorithms" by Jean Meeus
- Calculates solar elevation for any time and location
- Determines day/night periods based on sun position

**Color Ramp Module** ([colorramp.rs](src/colorramp.rs))
- Contains blackbody color table (1000K-25100K in 100K intervals)
- Interpolates between table values for precise temperatures
- Applies brightness and gamma correction

**Gamma Methods** ([gamma.rs](src/gamma.rs))
- Trait-based design for multiple adjustment methods
- Currently implements dummy method (prints temperature, no display changes)
- Ready for additional methods: DRM, RandR, VidMode

**Location Providers** ([location.rs](src/location.rs))
- Trait-based design for multiple location sources
- Currently implements manual provider (user-specified lat/lon)
- Ready for additional provider: GeoClue2

## Next Steps

To complete the rewrite, the following work remains:

1. **Real Gamma Methods** - Port Linux gamma adjustment methods:
   - DRM (Direct Rendering Manager for TTY/framebuffer)
   - RandR (X11 RandR extension, preferred, multi-output)
   - VidMode (X11 VidMode extension, legacy, single output)
2. **Additional Location Provider** - Port automatic location detection:
   - GeoClue2 (Linux location service)
3. **Configuration File Support** - Parse and apply INI-style config files
4. **Signal Handling** - Respond to SIGUSR1 (toggle), SIGINT/SIGTERM (restore & exit)
5. **Hook Scripts** - Execute user scripts on period changes

## Testing

The Rust rewrite has been tested and verified to:
- Parse command-line arguments correctly
- Calculate solar elevation accurately
- Determine day/night/transition periods
- Compute appropriate color temperatures
- Display verbose output with solar information
- Run continuously with smooth fade transitions
- Update temperature based on changing solar position

Example test (print mode - shows current status and exits):
```bash
$ ./target/debug/redshift-rebooted -l 12:-34 -m dummy -pv
Location: 12.00, -34.00
Period: Night
Color temperature: 3500K
Brightness: 1.00
Gamma: 1.00, 1.00, 1.00
Solar elevation: -44.03°
```

Example test (continual mode - runs forever with updates):
```bash
$ ./target/debug/redshift-rebooted -l 40:-74 -m dummy -v
Location: 40.00, -74.00
Period: Night
Color temperature: 3500K
# Smooth fade from initial 6500K to target 3500K over ~4 seconds
Temperature: 6494
Temperature: 6478
...
Temperature: 3500
# Then continues monitoring, sleeping 5 seconds between checks
```

### Unit Tests

Comprehensive test suites have been created for all non-dummy/non-placeholder code:

```bash
cargo test
```

**Test Coverage:**
- **types_tests.rs** (9 tests): Core type definitions, bounds checking, defaults
- **solar_tests.rs** (8 tests): Solar elevation calculations, time-based variations
- **colorramp_tests.rs** (13 tests): Color temperature conversions, gamma/brightness adjustments
- **location_tests.rs** (19 tests): Manual location provider functionality, option parsing
- **continual_mode_tests.rs** (24 tests): Event loop logic, transition progress, fade animations, color interpolation

**Total: 73 passing tests**

All tests verify correct behavior against the legacy C implementation, including:
- Solar position calculations at various latitudes/longitudes
- Color temperature interpolation from blackbody table
- Gamma ramp adjustments with brightness and gamma correction
- Location provider initialization and configuration
- Transition progress calculation from solar elevation
- Fade animation smoothness and easing functions
- Color setting interpolation and major difference detection
- Complete event loop iteration logic

## Compatibility

This rewrite maintains compatibility with the legacy C version's command-line interface for basic operations. The output format and calculation methods are designed to match the original implementation.
