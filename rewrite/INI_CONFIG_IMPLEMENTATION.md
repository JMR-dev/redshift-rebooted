# INI Configuration File Implementation

This document describes the INI configuration file parsing implementation for Redshift.

## Overview

The Rust rewrite now supports reading configuration files in the same INI format as the original C version. This provides backward compatibility and allows users to configure Redshift without command-line arguments.

## Implementation Details

### Files Added/Modified

1. **`src/config_ini.rs`** - New module for INI parsing
   - `RedshiftConfig` struct to hold all INI settings
   - Config file search logic (XDG directories)
   - Parsing functions for special formats (brightness, gamma, time ranges)

2. **`src/main.rs`** - Updated main program
   - Load INI config early in startup
   - Merge INI settings with CLI args (CLI takes priority)
   - Build transition scheme from merged settings
   - Support new CLI options: `-b/--brightness` and `-g/--gamma`

3. **`Cargo.toml`** - Added dependency
   - `rust-ini = "0.21"` for INI file parsing

4. **Test files:**
   - `tests/config_ini_tests.rs` - Unit tests for INI parsing
   - `tests/config_merging_tests.rs` - Integration tests for config merging

5. **Documentation:**
   - `CONFIG.md` - User-facing configuration guide
   - `redshift.conf.sample` - Sample configuration file

### Dependencies

- **rust-ini** (0.21): Mature, well-maintained INI parsing library
  - Simple API for reading INI files
  - Section-based organization matching our config format
  - Handles edge cases and malformed files gracefully

### Config File Search Order

The implementation searches for config files in these locations:

1. `$XDG_CONFIG_HOME/redshift/redshift.conf`
2. `$XDG_CONFIG_HOME/redshift.conf`
3. `$HOME/.config/redshift/redshift.conf`
4. `$HOME/.config/redshift.conf`
5. `/etc/redshift/redshift.conf`
6. `/etc/redshift.conf`

This matches the C version's behavior for maximum compatibility.

### Supported Settings

#### `[redshift]` Section

| Setting | Format | Example | Description |
|---------|--------|---------|-------------|
| `temp-day` | Integer | `5700` | Day temperature in Kelvin |
| `temp-night` | Integer | `3500` | Night temperature in Kelvin |
| `fade` / `transition` | 0 or 1 | `1` | Enable smooth transitions |
| `brightness` | Float or Float:Float | `0.9` or `0.7:0.4` | Brightness (day:night) |
| `brightness-day` | Float | `0.9` | Day brightness (0.1-1.0) |
| `brightness-night` | Float | `0.7` | Night brightness (0.1-1.0) |
| `gamma` | Float or R:G:B | `0.8` or `0.8:0.7:0.9` | Gamma for all or RGB |
| `gamma-day` | Float or R:G:B | `0.8:0.7:0.9` | Day gamma |
| `gamma-night` | Float or R:G:B | `0.6` | Night gamma |
| `elevation-high` | Float | `3.0` | Solar elevation for day (degrees) |
| `elevation-low` | Float | `-6.0` | Solar elevation for night (degrees) |
| `dawn-time` | HH:MM[-HH:MM] | `6:00-7:45` | Dawn time range |
| `dusk-time` | HH:MM[-HH:MM] | `18:35-20:15` | Dusk time range |
| `location-provider` | String | `manual` | Location provider |
| `adjustment-method` | String | `randr` | Gamma method |

#### `[manual]` Section

| Setting | Format | Example | Description |
|---------|--------|---------|-------------|
| `lat` | Float | `40.7` | Latitude (-90 to 90) |
| `lon` | Float | `-74.0` | Longitude (-180 to 180) |

#### `[randr]` Section

| Setting | Format | Example | Description |
|---------|--------|---------|-------------|
| `screen` | Integer | `0` | X11 screen number |
| `crtc` | Integer | `0` | Specific CRTC ID |

### Parsing Functions

#### Brightness Parsing
```rust
parse_brightness_string(s: &str) -> Result<(f32, f32), String>
```
- Single value: `"0.9"` → `(0.9, 0.9)`
- Separate values: `"0.7:0.4"` → `(0.7, 0.4)`

#### Gamma Parsing
```rust
parse_gamma_string(s: &str) -> Result<[f32; 3], String>
```
- Single value: `"0.8"` → `[0.8, 0.8, 0.8]`
- RGB values: `"0.8:0.7:0.9"` → `[0.8, 0.7, 0.9]`

#### Time Range Parsing
```rust
parse_time_range(s: &str) -> Result<TimeRange, String>
```
- Single time: `"6:00"` → `TimeRange { start: 21600, end: 21600 }`
- Range: `"6:00-7:45"` → `TimeRange { start: 21600, end: 27900 }`

Times are stored as seconds since midnight.

### Merging Strategy

Settings are applied with this priority:

1. **Command-line arguments** (highest)
2. **INI configuration file**
3. **Built-in defaults** (lowest)

Example:
```bash
# Config file has temp-day=5700
# Running with: redshift -t 6000
# Result: Uses 6000K (CLI override)
```

Location is also merged with priority:

1. CLI location (`-l LAT:LON`)
2. INI manual location (`[manual]` section)
3. Saved TOML location (from previous runs)
4. GeoClue2 automatic detection
5. Interactive selection

### Validation

All settings are validated against bounds:

- Temperature: 1000-25000 K
- Brightness: 0.1-1.0
- Gamma: 0.1-10.0
- Latitude: -90 to 90
- Longitude: -180 to 180

Invalid values result in error messages and the program exits.

## Testing

### Unit Tests

Located in `src/config_ini.rs`:
- `test_parse_brightness_single`
- `test_parse_brightness_separate`
- `test_parse_gamma_single`
- `test_parse_gamma_rgb`
- `test_parse_time`
- `test_parse_time_range`
- `test_parse_time_range_single`

### Integration Tests

**`tests/config_ini_tests.rs`**:
- Full config file parsing
- Individual section parsing
- Error handling for malformed configs
- Path search logic
- Config file not found scenarios

**`tests/config_merging_tests.rs`**:
- Temperature loading
- Brightness loading (separate day/night)
- Gamma loading (separate day/night)
- Elevation settings
- Time-based transitions
- All sections together

All tests pass successfully.

### Manual Testing

Verified with sample config:
```bash
env HOME=/tmp/redshift-test ./target/release/redshift-rebooted -pv
```

Output correctly shows:
- Location from INI config
- Temperature from config
- Brightness from config
- Gamma from config
- CLI overrides work correctly

## Backward Compatibility

The implementation is fully compatible with the C version:

1. **Same file format**: Standard INI with sections
2. **Same search paths**: XDG directories, fallbacks, etc.
3. **Same section names**: `[redshift]`, `[manual]`, `[randr]`
4. **Same setting names**: All original settings supported
5. **Same aliases**: Both `fade` and `transition` work

Existing `redshift.conf` files work without modification.

## Usage Examples

### Basic Configuration

```ini
[redshift]
temp-day=5700
temp-night=3500

[manual]
lat=40.7
lon=-74.0
```

### Advanced Configuration

```ini
[redshift]
temp-day=5500
temp-night=3200
brightness-day=1.0
brightness-night=0.75
gamma-day=0.9:0.85:0.95
gamma-night=0.7
elevation-high=5
elevation-low=-8
fade=1

[manual]
lat=51.5074
lon=-0.1278
```

### Time-Based Transitions

```ini
[redshift]
temp-day=6500
temp-night=3500
dawn-time=6:00-7:45
dusk-time=18:35-20:15

[manual]
lat=48.1
lon=11.6
```

## Command-Line Overrides

All config settings can be overridden:

```bash
# Override temperature
redshift -t 6000 --temp-night 4000

# Override brightness
redshift -b 0.8:0.6

# Override gamma
redshift -g 0.9

# Override with RGB gamma
redshift -g 1.0:0.9:1.0

# Override location
redshift -l 40.7:-74.0
```

## Future Enhancements

Potential improvements for future versions:

1. Support for additional gamma methods (DRM, VidMode, etc.)
2. Per-screen/CRTC brightness and gamma settings
3. Hook scripts configuration
4. Multiple location providers in config
5. Configuration validation with detailed error messages
6. Config file generation from CLI settings

## References

- Original C implementation: `legacy/src/config-ini.c`
- Sample config: `legacy/redshift.conf.sample`
- rust-ini crate: https://crates.io/crates/rust-ini
