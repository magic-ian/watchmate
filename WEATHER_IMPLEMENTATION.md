# Weather Provider Selector Implementation

This document describes the implementation of the weather provider selector feature for Watchmate.

## Overview

The weather provider selector allows Watchmate to detect and interface with weather applications (KDE Weather and GNOME Weather) and send weather data to the InfiniTime smartwatch, similar to how Gadgetbridge works on Android.

## Architecture

The implementation follows the same pattern as the existing media player selector:

### 1. InfiniTime Bluetooth Service Layer (`infinitime/src/bluetooth/`)

#### Weather Service UUIDs (`uuids.rs`)
- `SRV_WEATHER`: Service UUID `00050000-78fc-48fe-8e23-433b3a1942d0`
- `CHR_WEATHER_DATA`: Characteristic UUID `00050001-78fc-48fe-8e23-433b3a1942d0`

#### Weather Device Interface (`device/weather.rs`)
Provides the following types and functions:

**Types:**
- `WeatherIcon`: Enum representing weather conditions (Sun, Clouds, Rain, etc.)
- `CurrentWeather`: Structure for current weather data
  - timestamp (i64)
  - temperature (i16, in °C * 100)
  - min_temperature (i16, in °C * 100)
  - max_temperature (i16, in °C * 100)
  - location (String, max 32 bytes)
  - icon (WeatherIcon)
  - sunrise (Option<i16>, minutes since midnight)
  - sunset (Option<i16>, minutes since midnight)
- `DayForecast`: Structure for a single day forecast
- `WeatherForecast`: Structure for multi-day forecast (up to 5 days)

**Methods:**
- `InfiniTime::send_current_weather(&self, weather: &CurrentWeather) -> Result<()>`
- `InfiniTime::send_weather_forecast(&self, forecast: &WeatherForecast) -> Result<()>`

These methods encode the weather data according to the InfiniTime SimpleWeatherService protocol and send it via Bluetooth.

### 2. Weather Provider Interface (`infinitime/src/freedesktop/weather.rs`)

Provides D-Bus integration for detecting and communicating with weather applications:

**Types:**
- `WeatherProvider`: Represents a weather application
  - name: Display name (e.g., "KDE Weather")
  - service_name: D-Bus service name (e.g., "org.kde.kweather")
- `ProvidersListEvent`: Enum for provider lifecycle events
  - ProviderAdded
  - ProviderRemoved

**Functions:**
- `get_providers(connection: &Connection) -> Result<Vec<WeatherProvider>>`: Returns list of available weather providers
- `check_service_exists(connection: &Connection, service_name: &str) -> Result<bool>`: Checks if a D-Bus service exists
- `get_current_weather(connection: &Connection, provider: &WeatherProvider) -> Result<CurrentWeather>`: Fetches current weather (stub)
- `get_forecast(connection: &Connection, provider: &WeatherProvider) -> Result<WeatherForecast>`: Fetches forecast (stub)
- `get_providers_update_stream(connection: &Arc<Connection>) -> Result<impl Stream<Item = ProvidersListEvent>>`: Monitors provider lifecycle

Currently monitors these D-Bus services:
- `org.kde.kweather` (KDE Weather)
- `org.gnome.Weather` (GNOME Weather)

### 3. UI Component (`watchmate/src/ui/dashboard_page/weather.rs`)

Implements the weather provider selector UI using Relm4:

**Component Structure:**
- Similar to media_player.rs
- Dropdown selector showing available weather providers
- Shows "Not available" when no providers are running
- Automatically detects when providers start/stop

**Input Messages:**
- `Device`: Connected/disconnected InfiniTime device
- `WeatherSessionStart`: Start weather update session
- `WeatherSessionEnded`: Weather session ended
- `ProviderUpdateSessionStart`: Start monitoring providers
- `ProviderUpdateSessionEnded`: Provider monitoring ended
- `ProviderAdded`: New weather provider detected
- `ProviderRemoved`: Weather provider stopped

**State:**
- `provider_handles`: List of available weather providers
- `provider_names`: GTK StringList for dropdown
- `infinitime`: Connected device reference
- `weather_task`: Handle for weather update task
- `update_task`: Handle for provider monitoring task
- `dbus_session`: D-Bus connection

### 4. Dashboard Integration (`watchmate/src/ui/dashboard_page.rs`)

The weather selector is integrated into the "Host Integration" section of the dashboard, positioned between the media player and notifications selectors.

## Protocol Details

The implementation follows the InfiniTime SimpleWeatherService protocol v1:

### Current Weather Message Format (53 bytes)
```
[0]       : Message type (0 = current weather)
[1]       : Message version (1 = with sunrise/sunset)
[2-9]     : Timestamp (64-bit Unix timestamp, little-endian)
[10-11]   : Current temperature (°C * 100, little-endian)
[12-13]   : Min temperature (°C * 100, little-endian)
[14-15]   : Max temperature (°C * 100, little-endian)
[16-47]   : Location (32-byte null-terminated string)
[48]      : Icon ID (0-8)
[49-50]   : Sunrise (minutes since midnight, little-endian, -1 for unknown)
[51-52]   : Sunset (minutes since midnight, little-endian, -1 for unknown)
```

### Forecast Message Format (36 bytes)
```
[0]       : Message type (1 = forecast)
[1]       : Message version (0)
[2-9]     : Timestamp (64-bit Unix timestamp, little-endian)
[10]      : Number of days (max 5)
[11-35]   : 5 days × 5 bytes each:
              [0-1] : Min temperature (°C * 100, little-endian)
              [2-3] : Max temperature (°C * 100, little-endian)
              [4]   : Icon ID
```

## Current Status

### Completed ✓
1. Weather service UUID definitions
2. Weather data structures and encoding
3. Bluetooth communication methods
4. D-Bus provider detection infrastructure
5. Provider lifecycle monitoring
6. UI component with provider selector
7. Dashboard integration

### Needs Implementation
1. **Weather Data Fetching**: The `get_current_weather()` and `get_forecast()` functions are stubs that need to be implemented to actually query weather data from KDE Weather and GNOME Weather via D-Bus
2. **Periodic Updates**: The weather session should periodically fetch and send weather data to the watch (e.g., every 30 minutes)
3. **D-Bus Interface Discovery**: Need to research and implement the actual D-Bus interfaces for:
   - KDE Weather (org.kde.kweather)
   - GNOME Weather (org.gnome.Weather)
4. **Error Handling**: Proper error handling for weather data conversion and transmission
5. **Settings**: User preferences for update frequency, temperature units, etc.
6. **Testing**: Full testing with actual weather applications

## Implementation Notes

### D-Bus Service Detection

The implementation handles different installation methods (binary, Flatpak, AppImage, Snap) by monitoring D-Bus service names. The actual D-Bus service name should be the same regardless of installation method, as it's the application identifier.

For Flatpaks specifically, the service name might be prefixed (e.g., `org.kde.kweather` vs `org.flatpak.kweather`), so this may need to be adjusted based on actual testing.

### Weather Data Source

Currently, the implementation assumes weather apps expose their data via D-Bus. If they don't, alternative approaches might be needed:
- Direct file/config reading
- HTTP API integration
- Integration with weather services (OpenWeatherMap, etc.)

### Temperature Units

The InfiniTime protocol expects temperatures in Celsius × 100 (to preserve decimal precision). The implementation will need to handle conversion if weather providers return data in different formats.

## Next Steps for Completion

1. **Research D-Bus Interfaces**:
   ```bash
   # For KDE Weather
   qdbus org.kde.kweather
   
   # For GNOME Weather
   gdbus introspect --session --dest org.gnome.Weather --object-path /org/gnome/Weather
   ```

2. **Implement Weather Data Fetching**:
   - Create D-Bus proxy interfaces for each weather provider
   - Parse weather data into CurrentWeather/WeatherForecast structures
   - Handle missing/invalid data gracefully

3. **Add Periodic Updates**:
   - Implement timer-based weather updates in the weather session
   - Make update interval configurable

4. **Testing**:
   - Test with KDE Weather installed
   - Test with GNOME Weather installed
   - Test with both installed
   - Test with neither installed
   - Test Flatpak installations

## References

- [InfiniTime SimpleWeatherService Documentation](https://github.com/InfiniTimeOrg/InfiniTime/blob/main/doc/SimpleWeatherService.md)
- [KDE Weather](https://invent.kde.org/utilities/kweather)
- [GNOME Weather](https://gitlab.gnome.org/GNOME/gnome-weather)
- [Gadgetbridge Weather Support](https://github.com/Freeyourgadget/Gadgetbridge)
- [Phosh Anti-Spam D-Bus Integration Reference](https://github.com/sc0w/phosh-antispam)
