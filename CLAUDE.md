# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Redshift adjusts screen color temperature according to time of day using gamma ramps. The project consists of:
- **redshift**: C program that manages color temperature adjustment
- **redshift-gtk**: Python GUI wrapper providing a system tray status icon

## Build System

This project uses GNU Autotools (autoconf/automake).

### Building from Source

```shell
# Initial setup (only needed after git clone)
./bootstrap

# Configure with prefix for local development
./configure --prefix=$HOME/redshift/root \
  --with-systemduserunitdir=$HOME/.config/systemd/user

# Build
make

# Install to prefix directory
make install

# Run distribution checks
make distcheck
```

### Platform-Specific Configuration

**Linux (full features):**
```shell
./configure --enable-drm --enable-vidmode --enable-randr --enable-geoclue2 --enable-gui --enable-apparmor
```

**macOS:**
```shell
./configure --enable-corelocation --enable-quartz --enable-gui
```

**Windows (cross-compile with MinGW):**
```shell
./configure --disable-drm --disable-randr --disable-vidmode --enable-wingdi \
  --disable-quartz --disable-geoclue2 --disable-corelocation --disable-gui \
  --disable-ubuntu --host=x86_64-w64-mingw32
```

### Running Tests

```shell
# Basic functionality test
./root/bin/redshift -l 12:-34 -pv

# Test with dummy adjustment method
./root/bin/redshift -l 12:-34 -m dummy -vo

# Test with configuration file
echo -e "[redshift]\ndawn-time=6:30\ndusk-time=18:00-19:30" > test.config
./root/bin/redshift -c test.config -pv
```

## Architecture

### Core Components

**Main Program (`src/redshift.c`):**
- Entry point and main event loop
- Sets display gamma every few seconds/minutes based on solar calculations
- Handles SIGUSR1 (toggle day/night), SIGINT/SIGTERM (restore gamma and exit)
- Short transitions (~10s) at startup/signal; long transitions (~50min) otherwise

**Adjustment Methods** (gamma ramp manipulation):
- `gamma-drm.c/h`: DRM method for Linux TTY
- `gamma-randr.c/h`: RANDR method (preferred for X11, supports multiple outputs)
- `gamma-vidmode.c/h`: VidMode method (X11, older, single output)
- `gamma-quartz.c/h`: macOS Quartz
- `gamma-w32gdi.c/h`: Windows GDI
- `gamma-dummy.c/h`: No-op for testing

**Location Providers** (determine geographic coordinates):
- `location-manual.c/h`: Manual lat/lon specification
- `location-geoclue2.c/h`: Automatic location via GeoClue2 (Linux)
- `location-corelocation.m/h`: macOS CoreLocation (Objective-C)

**Supporting Modules:**
- `solar.c/h`: Solar position calculations for day/night timing
- `colorramp.c/h`: Color temperature to RGB gamma conversion
- `config-ini.c/h`: Configuration file parsing
- `options.c/h`: Command-line option handling
- `hooks.c/h`: Hook scripts for events
- `signals.c/h`: Unix signal handling
- `systemtime.c/h`: System time utilities

### GUI Component

**redshift-gtk** (Python):
- Located in `src/redshift-gtk/`
- `controller.py`: Manages redshift subprocess
- `statusicon.py`: System tray icon
- `utils.py`: Utility functions
- Sends SIGUSR1 to redshift when user clicks icon

### Conditional Compilation

Features are enabled/disabled via autoconf (`configure.ac`). The `src/Makefile.am` conditionally includes source files based on:
- `ENABLE_DRM`, `ENABLE_RANDR`, `ENABLE_VIDMODE`, `ENABLE_QUARTZ`, `ENABLE_WINGDI`
- `ENABLE_GEOCLUE2`, `ENABLE_CORELOCATION`
- `ENABLE_GUI`, `ENABLE_UBUNTU`, `ENABLE_SYSTEMD`, `ENABLE_APPARMOR`

## C Coding Style (from CONTRIBUTING.md)

- Follow Linux kernel coding style
- Max 80 characters per line in new code
- All structures are typedef'd
- No Yoda conditions
- No multiline if-statements without braces (use single line or add braces)
- C-style comments only (`/* */`)

## Dependencies

**Build tools:**
- autotools, gettext, intltool, libtool

**Optional runtime (Linux):**
- libdrm (DRM support)
- libxcb, libxcb-randr (RandR support)
- libX11, libXxf86vm (VidMode support)
- glib-2.0, gio-2.0 >= 2.26 (GeoClue2 support)

**Optional GUI:**
- python3 >= 3.2, pygobject, pyxdg
- appindicator (Ubuntu-style status icon)

See `.travis.yml` for Ubuntu package list.

## Configuration

Configuration file location:
- **Linux/macOS**: `~/.config/redshift.conf`
- **Windows**: `%USERPROFILE%\AppData\Local\redshift.conf`

Sample: `redshift.conf.sample`

## Location Provider Syntax

Command-line location syntax: `-l PROVIDER:OPTIONS`

Special case: `-l LAT:LON` (parsed as manual provider when LAT is numeric)

Example: `-l manual:lat=55:lon=12` is equivalent to `-l 55:12`

Longitudes in western hemisphere must be negative (e.g., New York: `41,-74`)

## Translation Updates

```shell
make update-po
```

Translations are managed via [Launchpad Translations for Redshift](https://translations.launchpad.net/redshift).

## Notes

- Verbose logging is controlled in `redshift.c`; all verbose messages should be written there
- Location providers are only queried at startup, not during runtime
- Gamma ramps are the mechanism for color adjustment; other applications (games, video players) may temporarily reset them
- Wayland and Mir are not supported; users should use desktop environment's built-in night light features (GNOME Night Light, KDE Night Color)
