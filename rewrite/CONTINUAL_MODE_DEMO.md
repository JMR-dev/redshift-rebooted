# Continual Mode Demonstration

This document demonstrates the continual mode functionality of the Rust rewrite.

## How It Works

Continual mode implements the main event loop that:
1. Continuously monitors solar position based on time and location
2. Calculates appropriate color temperature for current conditions
3. Applies smooth fade transitions when temperature changes significantly
4. Sleeps intelligently: 100ms during fades, 5 seconds during stable periods

## Key Implementation Details

### Timing Constants
- `SLEEP_DURATION = 5000ms` - Normal sleep between updates
- `SLEEP_DURATION_SHORT = 100ms` - Sleep during fade animations
- `FADE_LENGTH = 40` - Number of steps in a fade animation

### Fade Logic
Fades are triggered when:
- Temperature difference > 25K
- Brightness difference > 0.1
- Any gamma channel difference > 0.1

The fade uses cubic easing: `f(t) = t² * (3 - 2t)` for smooth visual transitions.

### Update Cycle
```
Loop:
  1. Get current time
  2. Calculate solar elevation
  3. Determine period (Day/Night/Transition)
  4. Calculate target color temperature
  5. Check if fade needed
  6. Apply fade step if active, or jump to target
  7. Set display temperature
  8. Sleep (short if fading, long if stable)
```

## Example Output

### Night Period (New York at night)
```bash
$ ./target/debug/redshift-rebooted -l 40:-74 -m dummy -v
Location: 40.00, -74.00
Period: Night
Color temperature: 3500K
# Smooth transition from 6500K → 3500K
Temperature: 6494
Temperature: 6478
...
Temperature: 3500
# Then stable at 3500K
```

### Transition Period
During sunrise/sunset, the verbose output shows transition progress:
```
Period: Transition (23.4%)
Color temperature: 4200K
```

## Testing Recommendations

1. **Test different locations and times** to verify solar calculations
2. **Observe fade smoothness** - should take ~4 seconds (40 steps × 100ms)
3. **Check stable periods** - should sleep 5 seconds between checks
4. **Monitor period changes** - verbose mode shows when day/night boundaries crossed

## Comparison with C Version

The Rust implementation matches the C version's behavior:
- Same fade duration (40 × 100ms = 4 seconds)
- Same sleep intervals (5s normal, 100ms during fade)
- Same transition logic based on solar elevation
- Same cubic easing function for smooth fades
