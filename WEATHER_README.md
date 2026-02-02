# Weather Provider Selector - Quick Reference

## Overview

The weather provider selector allows Watchmate to detect weather applications (KDE Weather and GNOME Weather) on your Linux system and send weather data to your InfiniTime smartwatch.

## Status: MVP Infrastructure Complete ✓

This implementation provides the foundation for weather provider support:

✅ **Completed:**
- Bluetooth protocol implementation (InfiniTime SimpleWeatherService)
- D-Bus provider detection (KDE Weather, GNOME Weather)
- UI component with provider dropdown selector
- Automatic provider lifecycle monitoring
- Dashboard integration

🚧 **TODO:**
- Implement actual weather data fetching from D-Bus
- Add periodic weather updates
- Add user configuration (update frequency, units, etc.)

## Quick Start

### For Users

1. Install a weather app:
   ```bash
   flatpak install org.kde.kweather
   # or
   flatpak install org.gnome.Weather
   ```

2. Build and run Watchmate:
   ```bash
   cargo run --release
   ```

3. Connect your InfiniTime watch

4. The weather provider selector will appear in the Dashboard under "Host Integration"

### For Developers

See detailed documentation:
- [WEATHER_IMPLEMENTATION.md](WEATHER_IMPLEMENTATION.md) - Architecture and implementation details
- [WEATHER_TESTING.md](WEATHER_TESTING.md) - Testing guide and troubleshooting
- [infinitime/examples/weather.rs](infinitime/examples/weather.rs) - Usage examples

## Architecture

```
┌─────────────────────────────────────┐
│   Watchmate UI (GTK/Relm4)          │
│   - Weather Provider Selector       │
│   - Dropdown with available apps    │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│   D-Bus Provider Interface          │
│   - Detect KDE Weather              │
│   - Detect GNOME Weather            │
│   - Monitor lifecycle               │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│   Weather Data Encoding             │
│   - CurrentWeather struct           │
│   - WeatherForecast struct          │
│   - Temperature conversion          │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│   Bluetooth (InfiniTime Protocol)   │
│   - Service: 00050000-...           │
│   - Characteristic: 00050001-...    │
└────────────┬────────────────────────┘
             │
             ▼
        ┌────────┐
        │ Watch  │
        └────────┘
```

## API Example

```rust
use infinitime::bt::device::weather::*;

// Create weather data
let weather = CurrentWeather {
    timestamp: chrono::Utc::now().timestamp(),
    temperature: 2250,  // 22.50°C
    min_temperature: 1800,
    max_temperature: 2600,
    location: "San Francisco".to_string(),
    icon: WeatherIcon::FewClouds,
    sunrise: Some(6 * 60 + 30),  // 6:30 AM
    sunset: Some(18 * 60 + 45),  // 6:45 PM
};

// Send to watch
device.send_current_weather(&weather).await?;
```

## Supported Weather Apps

| App | D-Bus Service | Status |
|-----|--------------|--------|
| KDE Weather | `org.kde.kweather` | Detected ✓ |
| GNOME Weather | `org.gnome.Weather` | Detected ✓ |

Works with any installation method: native packages, Flatpak, Snap, AppImage.

## InfiniTime Compatibility

Requires InfiniTime firmware **1.14.0 or later** for SimpleWeatherService support.

## Data Protocol

The implementation follows the InfiniTime SimpleWeatherService protocol v1:

- **Current Weather**: 53 bytes
  - Timestamp, temperatures (current/min/max)
  - Location (32 chars max)
  - Weather icon (9 types)
  - Sunrise/sunset times

- **Forecast**: 36 bytes
  - 5-day forecast
  - Min/max temperatures per day
  - Weather icon per day

Temperatures are in Celsius × 100 (e.g., 22.5°C = 2250).

## Future Enhancements

Potential improvements for future versions:

1. **Direct API Integration**
   - OpenWeatherMap API support
   - Weather.gov API support
   - Custom weather source configuration

2. **Advanced Features**
   - Location selection
   - Multiple location support
   - Weather alerts/warnings
   - Precipitation probability
   - Wind speed/direction
   - Humidity, pressure, UV index (if InfiniTime adds support)

3. **User Configuration**
   - Update frequency (15min, 30min, 1hr, etc.)
   - Temperature units (C/F)
   - Auto-detect location
   - Provider priority

4. **Integrations**
   - Geoclue integration for location
   - Network location services
   - Time zone awareness

## Contributing

To contribute to weather provider support:

1. Test with your weather app installation
2. Report issues with specific D-Bus service names
3. Submit PRs for additional weather providers
4. Help implement weather data fetching

See [CONTRIBUTING.md](../CONTRIBUTING.md) if available.

## References

- [InfiniTime SimpleWeatherService](https://github.com/InfiniTimeOrg/InfiniTime/blob/main/doc/SimpleWeatherService.md)
- [KDE Weather](https://invent.kde.org/utilities/kweather)
- [GNOME Weather](https://gitlab.gnome.org/GNOME/gnome-weather)
- [Gadgetbridge Weather](https://github.com/Freeyourgadget/Gadgetbridge)

## License

Same as the parent Watchmate project.
