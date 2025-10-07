# Configuration File Support

Redshift supports configuration files in INI format, compatible with the original C version.

## Configuration File Locations

Redshift searches for configuration files in the following locations (in order of priority):

1. `$XDG_CONFIG_HOME/redshift/redshift.conf`
2. `$XDG_CONFIG_HOME/redshift.conf` (fallback)
3. `$HOME/.config/redshift/redshift.conf`
4. `$HOME/.config/redshift.conf` (fallback)
5. `/etc/redshift/redshift.conf` (system-wide)
6. `/etc/redshift.conf` (system-wide fallback)

## Configuration Sections

### `[redshift]` - Main Settings

**Temperature settings:**
- `temp-day` - Day color temperature in Kelvin (default: 6500)
- `temp-night` - Night color temperature in Kelvin (default: 3500)

**Transition settings:**
- `fade` or `transition` - Smooth transition (0=off, 1=on, default: 1)
- `elevation-high` - Solar elevation for day in degrees (default: 3.0)
- `elevation-low` - Solar elevation for night in degrees (default: -6.0)

**Time-based transitions (alternative to elevation):**
- `dawn-time` - Dawn time range, e.g., `6:00-7:45`
- `dusk-time` - Dusk time range, e.g., `18:35-20:15`

**Brightness settings:**
- `brightness` - Single value for both day and night (0.1-1.0)
- `brightness-day` - Day brightness (0.1-1.0)
- `brightness-night` - Night brightness (0.1-1.0)

**Gamma settings:**
- `gamma` - Single value for all RGB channels, or R:G:B format
  - Example: `gamma=0.8` (applies 0.8 to all channels)
  - Example: `gamma=0.8:0.7:0.9` (R=0.8, G=0.7, B=0.9)
- `gamma-day` - Day gamma value(s)
- `gamma-night` - Night gamma value(s)

**Provider/Method settings:**
- `location-provider` - Location provider (manual, geoclue2)
- `adjustment-method` - Gamma adjustment method (randr, dummy)

### `[manual]` - Manual Location

- `lat` - Latitude (-90 to 90)
- `lon` - Longitude (-180 to 180)

**Note:** Longitudes west of Greenwich (e.g., Americas) are negative.

### `[randr]` - RandR Method Settings

- `screen` - X11 screen number to adjust (default: all screens)
- `crtc` - Specific CRTC to adjust (optional)

## Priority Order

Settings are applied in the following priority order (highest to lowest):

1. **Command-line arguments** (highest priority)
2. **INI configuration file**
3. **Default values** (lowest priority)

### Examples

If you specify `-t 6000` on the command line and `temp-day=5700` in the config file, the command-line value (6000K) will be used.

## Sample Configuration

```ini
[redshift]
; Color temperature
temp-day=5700
temp-night=3500

; Enable smooth transitions
fade=1

; Brightness (optional)
brightness-day=0.9
brightness-night=0.7

; Gamma correction (optional)
gamma=0.8
; Or per-channel: gamma=0.8:0.7:0.9

; Solar elevation thresholds (optional)
;elevation-high=3
;elevation-low=-6

; Or use time-based transitions (optional)
;dawn-time=6:00-7:45
;dusk-time=18:35-20:15

; Location and adjustment method
location-provider=manual
adjustment-method=randr

[manual]
lat=40.7
lon=-74.0

[randr]
screen=0
```

## Location Priority

Locations are determined in the following order:

1. **Command-line location** (`-l LAT:LON`)
2. **INI config manual location** (`[manual]` section)
3. **Saved TOML location** (from previous runs)
4. **GeoClue2 automatic detection** (if available)
5. **Interactive selection** (fallback)

## Command-Line Options

You can override any config file setting with command-line arguments:

```bash
# Override temperature from config
redshift -t 6000 --temp-night 4000

# Override brightness
redshift -b 0.8

# Override gamma
redshift -g 0.9

# Override location
redshift -l 40.7:-74.0

# Print current settings
redshift -p

# Verbose output (shows where settings come from)
redshift -v
```

## Compatibility

This implementation is compatible with the original Redshift configuration file format. Existing `redshift.conf` files should work without modification.
