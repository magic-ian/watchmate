# Weather Provider Integration Guide

This document describes how weather applications can integrate with Watchmate to provide weather data to InfiniTime watches.

## Overview

Watchmate supports forwarding weather data from desktop weather applications to InfiniTime-powered smartwatches. The integration follows the same pattern as media player control (MPRIS), using D-Bus for inter-process communication.

## Weather Data Format

InfiniTime uses the SimpleWeatherService protocol. Weather data includes:

### Current Weather
- **Temperature**: Current, minimum, and maximum (°C)
- **Location**: City/location name (max 32 characters)
- **Weather Icon**: One of 9 icon types (sun, clouds, rain, snow, etc.)
- **Sunrise/Sunset**: Optional times in minutes since midnight
- **Timestamp**: When the data was retrieved

### Forecast
- **Up to 5 days** of forecast data
- Each day includes: min/max temperature and weather icon
- **Timestamp**: When the forecast was retrieved

## D-Bus Interface Requirements

### For Weather Application Developers

To integrate your weather application with Watchmate, you need to expose weather data via D-Bus. Here's what's needed:

#### 1. D-Bus Service Registration

Register your service on the session bus with a well-known name:
- **KDE Weather**: `org.kde.kweather` (recommended)
- **GNOME Weather**: `org.gnome.Weather.Application` (recommended)
- **Custom apps**: `org.example.YourWeatherApp`

#### 2. Recommended D-Bus Interface

While Watchmate can be adapted to work with any D-Bus interface, we recommend implementing a standard interface for weather data:

**Interface Name**: `org.freedesktop.Weather`

**Object Path**: `/org/freedesktop/Weather`

**Methods**:

```xml
<interface name="org.freedesktop.Weather">
  <!-- Get current weather data -->
  <method name="GetCurrentWeather">
    <arg name="weather" type="a{sv}" direction="out"/>
    <!--
      Dictionary containing:
        - location: string (city name)
        - temperature: double (°C)
        - min_temperature: double (°C)
        - max_temperature: double (°C)
        - icon_code: string (see icon mapping below)
        - timestamp: int64 (Unix timestamp)
        - sunrise: uint16 (optional, minutes since midnight)
        - sunset: uint16 (optional, minutes since midnight)
    -->
  </method>

  <!-- Get forecast data -->
  <method name="GetForecast">
    <arg name="forecast" type="a{sv}" direction="out"/>
    <!--
      Dictionary containing:
        - timestamp: int64 (Unix timestamp)
        - days: array of dictionaries, each containing:
          - min_temperature: double (°C)
          - max_temperature: double (°C)
          - icon_code: string
    -->
  </method>
</interface>
```

**Signals** (optional but recommended):

```xml
  <!-- Emitted when weather data is updated -->
  <signal name="WeatherUpdated">
    <arg name="timestamp" type="x"/>
  </signal>
```

#### 3. Icon Code Mapping

Use descriptive icon codes that Watchmate can map to InfiniTime icons:

| Your Icon Code | InfiniTime Icon | Description |
|----------------|-----------------|-------------|
| `clear-sky`, `sun`, `sunny` | Sun | Clear sky |
| `few-clouds`, `partly-cloudy` | CloudsSun | Few clouds |
| `scattered-clouds`, `cloudy` | Clouds | Scattered clouds |
| `broken-clouds`, `overcast` | BrokenClouds | Heavy clouds |
| `shower-rain`, `light-rain` | CloudShowerHeavy | Shower rain |
| `rain`, `moderate-rain` | CloudSunRain | Rain |
| `thunderstorm`, `storm` | Thunderstorm | Thunderstorm |
| `snow` | Snow | Snow |
| `mist`, `fog`, `smog` | Smog | Mist/fog |

### For Watchmate Integration

Watchmate will:
1. **Discover** weather providers by monitoring D-Bus service registration
2. **Query** weather data using the provider's D-Bus interface
3. **Convert** weather data to InfiniTime's SimpleWeatherService format
4. **Send** weather data to the watch via Bluetooth

## Implementation Examples

### KDE Weather (kweather)

KDE Weather could expose its data via D-Bus as follows:

```cpp
// Example D-Bus adaptation layer for KDE Weather
class WeatherDBusInterface : public QObject {
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.freedesktop.Weather")

public slots:
    QVariantMap GetCurrentWeather() {
        // Return current weather data from kweather backend
        QVariantMap weather;
        weather["location"] = currentLocation();
        weather["temperature"] = currentTemperature();
        weather["min_temperature"] = todayMinTemp();
        weather["max_temperature"] = todayMaxTemp();
        weather["icon_code"] = currentWeatherIcon();
        weather["timestamp"] = QDateTime::currentSecsSinceEpoch();
        return weather;
    }

    QVariantMap GetForecast() {
        // Return forecast data
        QVariantMap forecast;
        forecast["timestamp"] = QDateTime::currentSecsSinceEpoch();
        forecast["days"] = getForecastDays(); // Returns array of forecast days
        return forecast;
    }

signals:
    void WeatherUpdated(qint64 timestamp);
};
```

### GNOME Weather

GNOME Weather could add D-Bus support:

```javascript
// Example D-Bus interface for GNOME Weather (GJS)
const WeatherInterface = `
<node>
  <interface name="org.freedesktop.Weather">
    <method name="GetCurrentWeather">
      <arg name="weather" type="a{sv}" direction="out"/>
    </method>
    <method name="GetForecast">
      <arg name="forecast" type="a{sv}" direction="out"/>
    </method>
  </interface>
</node>`;

// Implementation would query GWeather and return data
```

## Testing Your Integration

1. **Register your D-Bus service**
   ```bash
   # Check if your service is visible
   busctl --user list | grep -i weather
   ```

2. **Test method calls**
   ```bash
   # Test GetCurrentWeather method
   busctl --user call org.kde.kweather \
       /org/freedesktop/Weather \
       org.freedesktop.Weather \
       GetCurrentWeather
   ```

3. **Monitor with D-Feet**
   - Install `d-feet` (D-Bus debugger)
   - Connect to session bus
   - Verify your service and methods are visible

## Benefits for Users

When weather apps expose data via D-Bus:
- **Works with any installation method**: Binary, Flatpak, Snap, AppImage
- **Automatic discovery**: Watchmate detects running weather apps
- **User choice**: Pick preferred weather provider from dropdown
- **No configuration needed**: Just install and run

## Pull Request Checklist

When submitting PRs to weather applications:

- [ ] Add D-Bus interface implementation
- [ ] Register service on session bus
- [ ] Implement `GetCurrentWeather()` method
- [ ] Implement `GetForecast()` method
- [ ] Emit `WeatherUpdated` signal on data refresh
- [ ] Add D-Bus dependency to build system
- [ ] Update documentation
- [ ] Add runtime flag to enable/disable D-Bus export (optional)

## References

- [InfiniTime SimpleWeatherService](https://github.com/InfiniTimeOrg/InfiniTime/blob/main/doc/SimpleWeatherService.md)
- [D-Bus Specification](https://dbus.freedesktop.org/doc/dbus-specification.html)
- [MPRIS D-Bus Interface](https://specifications.freedesktop.org/mpris-spec/latest/) (similar pattern)
- [Phosh Antispam](https://github.com/sc0w/phosh-antispam) (example of D-Bus app integration)

## Contact

For questions or assistance with integration:
- Open an issue in the Watchmate repository
- Discuss in InfiniTime community channels
