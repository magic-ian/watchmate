# D-Bus Weather Interface Specification

This document specifies the D-Bus interfaces that weather applications (KDE Weather, GNOME Weather) should expose to enable integration with smartwatch companion apps like Watchmate.

## Overview

Weather applications should expose their current weather data and forecasts via D-Bus to allow other applications (like smartwatch companions) to access and forward this data to connected devices.

## Motivation

- **Smartwatch Integration**: Companion apps like Watchmate need access to weather data to send to smartwatches (e.g., InfiniTime on PineTime)
- **Desktop Integration**: Other desktop applications and widgets can display weather information
- **Automation**: Scripts and automation tools can react to weather conditions
- **Standardization**: A common interface allows multiple weather apps to be used interchangeably

## Use Cases

1. **Smartwatch Companion**: Watchmate monitors for weather app D-Bus services, retrieves current weather and forecasts, and sends to InfiniTime watch
2. **Desktop Widgets**: Third-party widgets can display weather without implementing their own backend
3. **Notifications**: Weather alert/warning systems
4. **Smart Home**: Automation based on weather conditions

## Design Principles

1. **Read-only Access**: The interface provides read-only access to weather data
2. **Passive Consumption**: Clients poll or subscribe to signals; they don't control the weather app
3. **Simple Types**: Use standard D-Bus types for maximum compatibility
4. **Optional Data**: Not all data may be available; use optional values
5. **Extensibility**: Design allows for future additions without breaking compatibility

## Interface Specification

### Service Names

- **KDE Weather**: `org.kde.kweather`
- **GNOME Weather**: `org.gnome.Weather`

### Object Path

- **KDE Weather**: `/org/kde/kweather`
- **GNOME Weather**: `/org/gnome/Weather`

### Interface Name

- **KDE Weather**: `org.kde.kweather.WeatherData`
- **GNOME Weather**: `org.gnome.Weather.WeatherData`

Note: Both should implement the same interface specification for maximum compatibility.

## Methods

### GetCurrentWeather

Returns current weather conditions.

**Signature**: `GetCurrentWeather() → a{sv}`

**Returns**: Dictionary with the following keys (all optional):

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `timestamp` | `x` (int64) | Unix timestamp of when data was fetched | `1706860800` |
| `temperature` | `d` (double) | Current temperature in Celsius | `22.5` |
| `temperatureMin` | `d` (double) | Minimum temperature (today) in Celsius | `18.0` |
| `temperatureMax` | `d` (double) | Maximum temperature (today) in Celsius | `26.0` |
| `location` | `s` (string) | Location name | `"San Francisco, CA"` |
| `weatherCode` | `s` (string) | Weather condition code | `"partly-cloudy"` |
| `weatherDescription` | `s` (string) | Human-readable description | `"Partly Cloudy"` |
| `humidity` | `u` (uint32) | Relative humidity percentage | `65` |
| `windSpeed` | `d` (double) | Wind speed in m/s | `5.5` |
| `windDirection` | `u` (uint32) | Wind direction in degrees | `270` |
| `pressure` | `u` (uint32) | Atmospheric pressure in hPa | `1013` |
| `sunrise` | `x` (int64) | Sunrise time (Unix timestamp) | `1706848800` |
| `sunset` | `x` (int64) | Sunset time (Unix timestamp) | `1706889600` |

**Weather Codes** (suggested standard):
- `"clear-sky"` or `"sunny"`
- `"partly-cloudy"` or `"few-clouds"`
- `"cloudy"` or `"overcast"`
- `"light-rain"` or `"drizzle"`
- `"rain"` or `"rainy"`
- `"thunderstorm"` or `"storm"`
- `"snow"` or `"snowy"`
- `"fog"` or `"mist"`
- `"heavy-clouds"`

### GetForecast

Returns weather forecast for upcoming days.

**Signature**: `GetForecast(i days) → aa{sv}`

**Parameters**:
- `days` (int32): Number of forecast days requested (typically 1-7)

**Returns**: Array of dictionaries, one per day, each with:

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `date` | `s` (string) | ISO 8601 date | `"2024-02-02"` |
| `timestamp` | `x` (int64) | Unix timestamp for midnight of that day | `1706832000` |
| `temperatureMin` | `d` (double) | Minimum temperature in Celsius | `15.0` |
| `temperatureMax` | `d` (double) | Maximum temperature in Celsius | `25.0` |
| `weatherCode` | `s` (string) | Weather condition code | `"rain"` |
| `weatherDescription` | `s` (string) | Human-readable description | `"Rainy"` |
| `precipitationChance` | `u` (uint32) | Probability of precipitation (%) | `80` |

### GetLocation

Returns the currently configured location.

**Signature**: `GetLocation() → s`

**Returns**: Location name as configured in the weather app

## Properties

### Location (read-only)

**Type**: `s` (string)

**Description**: Currently configured location name

### LastUpdate (read-only)

**Type**: `x` (int64)

**Description**: Unix timestamp of last successful weather data update

### IsValid (read-only)

**Type**: `b` (boolean)

**Description**: Whether current weather data is valid/available

## Signals

### WeatherUpdated

Emitted when weather data has been refreshed.

**Signature**: `WeatherUpdated()`

**Description**: Applications should re-fetch weather data when this signal is received.

## Error Handling

Methods should return D-Bus errors with appropriate error names:

- `org.freedesktop.DBus.Error.Failed`: General failure
- `org.freedesktop.DBus.Error.NoReply`: Weather service not responding
- `org.kde.kweather.Error.NoData`: No weather data available
- `org.kde.kweather.Error.LocationNotSet`: No location configured

## Example Usage

### From Command Line (busctl)

```bash
# Check if service exists
busctl --user list | grep kweather

# Get current weather
busctl --user call org.kde.kweather /org/kde/kweather \
  org.kde.kweather.WeatherData GetCurrentWeather

# Get 5-day forecast
busctl --user call org.kde.kweather /org/kde/kweather \
  org.kde.kweather.WeatherData GetForecast i 5

# Monitor for updates
busctl --user monitor org.kde.kweather
```

### From Python (pydbus)

```python
from pydbus import SessionBus

bus = SessionBus()
weather = bus.get('org.kde.kweather', '/org/kde/kweather')

# Get current weather
current = weather.GetCurrentWeather()
print(f"Temperature: {current.get('temperature', 'N/A')}°C")
print(f"Condition: {current.get('weatherDescription', 'Unknown')}")

# Get forecast
forecast = weather.GetForecast(5)
for day in forecast:
    print(f"{day['date']}: {day['temperatureMin']}-{day['temperatureMax']}°C")

# Listen for updates
def on_update():
    print("Weather data updated!")
    current = weather.GetCurrentWeather()
    # ... update UI ...

weather.onWeatherUpdated = on_update
```

### From Rust (zbus)

```rust
use zbus::{Connection, proxy};
use std::collections::HashMap;

#[proxy(
    interface = "org.kde.kweather.WeatherData",
    default_service = "org.kde.kweather",
    default_path = "/org/kde/kweather"
)]
trait WeatherData {
    fn get_current_weather(&self) -> zbus::Result<HashMap<String, zbus::zvariant::Value>>;
    fn get_forecast(&self, days: i32) -> zbus::Result<Vec<HashMap<String, zbus::zvariant::Value>>>;
    fn get_location(&self) -> zbus::Result<String>;
    
    #[zbus(signal)]
    fn weather_updated(&self) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let connection = Connection::session().await?;
    let proxy = WeatherDataProxy::new(&connection).await?;
    
    // Get current weather
    let current = proxy.get_current_weather().await?;
    if let Some(temp) = current.get("temperature") {
        println!("Temperature: {:?}", temp);
    }
    
    // Get forecast
    let forecast = proxy.get_forecast(5).await?;
    println!("5-day forecast: {} days", forecast.len());
    
    Ok(())
}
```

## Implementation Notes

### Data Freshness

- Weather data should be cached and refreshed periodically (e.g., every 30-60 minutes)
- Apps should not make network requests on every D-Bus method call
- Use the `LastUpdate` property to indicate data age
- Emit `WeatherUpdated` signal after successful refresh

### Privacy Considerations

- Location information is sensitive
- Weather apps should respect user privacy settings
- Consider not exposing precise coordinates, only city names
- Users should be aware that other apps can access their location via weather data

### Threading

- D-Bus method calls should not block the UI
- Implement async/non-blocking handlers
- Return cached data immediately if available

### Backwards Compatibility

- If adding new keys to dictionaries, existing clients will ignore them
- Never remove or rename existing keys
- Use optional values for data that might not be available

## Version History

- **Version 1.0** (2024): Initial specification
  - Basic current weather and forecast
  - Standard weather codes
  - Update signals

## Future Considerations

Potential future additions (without breaking compatibility):

- **Weather Alerts**: Method to get active weather warnings
- **Hourly Forecast**: More granular forecast data
- **Historical Data**: Access to past weather observations
- **Multi-location**: Support for multiple saved locations
- **Units Preference**: Return data in user's preferred units
- **Attribution**: Weather data source/provider information

## References

- [D-Bus Specification](https://dbus.freedesktop.org/doc/dbus-specification.html)
- [InfiniTime SimpleWeatherService](https://github.com/InfiniTimeOrg/InfiniTime/blob/main/doc/SimpleWeatherService.md)
- [OpenWeatherMap Icon Mapping](https://openweathermap.org/weather-conditions)
- [Gadgetbridge Weather Support](https://github.com/Freeyourgadget/Gadgetbridge)
