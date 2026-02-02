# KDE Weather D-Bus Interface - Pull Request Proposal

## Overview

This document describes the proposed changes to KDE Weather to expose weather data via D-Bus, enabling integration with smartwatch companion apps and other desktop utilities.

## Motivation

Currently, KDE Weather stores and displays weather data but doesn't expose it via D-Bus for other applications to consume. This limits integration opportunities:

- **Smartwatch Integration**: Companion apps like Watchmate need weather data to send to smartwatches (InfiniTime on PineTime)
- **Desktop Widgets**: Third-party widgets could display weather without implementing their own backend
- **Automation**: Scripts and smart home systems could react to weather conditions
- **Accessibility**: Other apps could provide alternative weather interfaces

## Benefits

1. **Ecosystem Integration**: Enables KDE Weather to be a weather data provider for the broader Linux desktop ecosystem
2. **Smartwatch Support**: Allows users to see weather on their smartwatches via companion apps
3. **No User Impact**: This is a pure addition - no existing functionality changes
4. **Privacy Preserved**: Only exposes what's already visible in the UI
5. **Standard Practice**: Follows D-Bus patterns used by other KDE apps (e.g., media players via MPRIS)

## Use Cases

### Primary: Smartwatch Integration

Watchmate (companion app for InfiniTime/PineTime smartwatch) would:
1. Detect KDE Weather via D-Bus service name
2. Fetch current weather and forecast
3. Send to connected smartwatch
4. Subscribe to updates for automatic refresh

### Secondary: Desktop Integration

- Plasma widgets showing weather without separate backend
- Shell scripts checking weather for automation
- Notification systems for weather alerts

## Technical Design

### D-Bus Interface

**Service Name**: `org.kde.kweather`  
**Object Path**: `/org/kde/kweather`  
**Interface Name**: `org.kde.kweather.WeatherData`

### Methods

#### GetCurrentWeather

```xml
<method name="GetCurrentWeather">
  <annotation name="org.qtproject.QtDBus.QtTypeName.Out0" value="QVariantMap"/>
  <arg name="weather" type="a{sv}" direction="out"/>
</method>
```

Returns a dictionary with:
- `timestamp` (int64): Unix timestamp
- `temperature` (double): Current temp in Celsius
- `temperatureMin` (double): Min temp today
- `temperatureMax` (double): Max temp today
- `location` (string): Location name
- `weatherCode` (string): Condition code (e.g., "partly-cloudy")
- `weatherDescription` (string): Human-readable description
- `sunrise` (int64): Sunrise Unix timestamp
- `sunset` (int64): Sunset Unix timestamp
- Additional optional fields (humidity, pressure, wind, etc.)

#### GetForecast

```xml
<method name="GetForecast">
  <arg name="days" type="i" direction="in"/>
  <annotation name="org.qtproject.QtDBus.QtTypeName.Out0" value="QVariantList"/>
  <arg name="forecast" type="aa{sv}" direction="out"/>
</method>
```

Returns array of dictionaries, one per forecast day.

#### GetLocation

```xml
<method name="GetLocation">
  <arg name="location" type="s" direction="out"/>
</method>
```

Returns configured location name.

### Properties

```xml
<property name="Location" type="s" access="read"/>
<property name="LastUpdate" type="x" access="read"/>
<property name="IsValid" type="b" access="read"/>
```

### Signals

```xml
<signal name="WeatherUpdated"/>
```

Emitted when weather data is refreshed.

## Implementation Approach

### File Structure

```
src/
├── weatherdatainterface.h      # D-Bus interface definition
├── weatherdatainterface.cpp    # D-Bus method implementations
└── weatherdataadaptor.xml      # Qt D-Bus XML definition
```

### Integration Points

1. **Main Application Class**: Register D-Bus service on startup
2. **Weather Data Model**: Connect to D-Bus adaptor
3. **Update Logic**: Emit `WeatherUpdated` signal after refresh
4. **Shutdown**: Unregister D-Bus service

### Code Sketch (Qt/C++)

```cpp
// weatherdatainterface.h
#ifndef WEATHERDATAINTERFACE_H
#define WEATHERDATAINTERFACE_H

#include <QObject>
#include <QDBusAbstractAdaptor>
#include <QVariantMap>
#include <QVariantList>

class WeatherDataInterface : public QDBusAbstractAdaptor
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.kde.kweather.WeatherData")
    
    Q_PROPERTY(QString Location READ location)
    Q_PROPERTY(qint64 LastUpdate READ lastUpdate)
    Q_PROPERTY(bool IsValid READ isValid)

public:
    explicit WeatherDataInterface(QObject *parent = nullptr);
    
    QString location() const;
    qint64 lastUpdate() const;
    bool isValid() const;

public slots:
    QVariantMap GetCurrentWeather();
    QVariantList GetForecast(int days);
    QString GetLocation();

signals:
    void WeatherUpdated();

private:
    QObject *m_weatherModel;
};

#endif
```

```cpp
// weatherdatainterface.cpp
#include "weatherdatainterface.h"
#include <QDBusConnection>
#include <QDateTime>

WeatherDataInterface::WeatherDataInterface(QObject *parent)
    : QDBusAbstractAdaptor(parent)
    , m_weatherModel(parent)
{
    // Connect to weather model updates
    connect(m_weatherModel, SIGNAL(weatherUpdated()), 
            this, SIGNAL(WeatherUpdated()));
}

QVariantMap WeatherDataInterface::GetCurrentWeather()
{
    QVariantMap weather;
    
    // Get data from weather model
    weather["timestamp"] = QDateTime::currentSecsSinceEpoch();
    weather["temperature"] = /* get from model */;
    weather["temperatureMin"] = /* get from model */;
    weather["temperatureMax"] = /* get from model */;
    weather["location"] = /* get from model */;
    weather["weatherCode"] = /* get from model */;
    weather["weatherDescription"] = /* get from model */;
    weather["sunrise"] = /* get from model */;
    weather["sunset"] = /* get from model */;
    
    return weather;
}

QVariantList WeatherDataInterface::GetForecast(int days)
{
    QVariantList forecast;
    
    // Get forecast data from model
    // Limit to requested number of days
    int maxDays = qMin(days, 7);
    
    for (int i = 0; i < maxDays; ++i) {
        QVariantMap day;
        // Populate from model
        day["date"] = /* ISO 8601 date */;
        day["timestamp"] = /* Unix timestamp */;
        day["temperatureMin"] = /* from model */;
        day["temperatureMax"] = /* from model */;
        day["weatherCode"] = /* from model */;
        day["weatherDescription"] = /* from model */;
        forecast.append(day);
    }
    
    return forecast;
}

QString WeatherDataInterface::GetLocation()
{
    return /* get from model */;
}

QString WeatherDataInterface::location() const
{
    return /* get from model */;
}

qint64 WeatherDataInterface::lastUpdate() const
{
    return /* get from model */;
}

bool WeatherDataInterface::isValid() const
{
    return /* check if model has valid data */;
}
```

```cpp
// In main application startup:
void KWeatherApp::registerDBus()
{
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    
    // Register service
    if (!sessionBus.registerService("org.kde.kweather")) {
        qWarning() << "Failed to register D-Bus service";
        return;
    }
    
    // Create and register adaptor
    m_dbusInterface = new WeatherDataInterface(m_weatherModel);
    
    if (!sessionBus.registerObject("/org/kde/kweather", 
                                    m_weatherModel,
                                    QDBusConnection::ExportAdaptors)) {
        qWarning() << "Failed to register D-Bus object";
        return;
    }
    
    qDebug() << "D-Bus interface registered successfully";
}
```

## Weather Code Mapping

Map internal weather condition codes to standard D-Bus codes:

```cpp
QString mapWeatherCode(const QString &internalCode)
{
    static const QMap<QString, QString> codeMap = {
        {"01d", "clear-sky"},
        {"01n", "clear-sky"},
        {"02d", "partly-cloudy"},
        {"02n", "partly-cloudy"},
        {"03d", "cloudy"},
        {"03n", "cloudy"},
        {"04d", "heavy-clouds"},
        {"04n", "heavy-clouds"},
        {"09d", "rain"},
        {"09n", "rain"},
        {"10d", "light-rain"},
        {"10n", "light-rain"},
        {"11d", "thunderstorm"},
        {"11n", "thunderstorm"},
        {"13d", "snow"},
        {"13n", "snow"},
        {"50d", "fog"},
        {"50n", "fog"},
    };
    
    return codeMap.value(internalCode, "unknown");
}
```

## Testing

### Manual Testing

```bash
# 1. Build and run KWeather with D-Bus support

# 2. Check service is registered
busctl --user list | grep kweather

# 3. Get current weather
busctl --user call org.kde.kweather /org/kde/kweather \
  org.kde.kweather.WeatherData GetCurrentWeather

# 4. Get forecast
busctl --user call org.kde.kweather /org/kde/kweather \
  org.kde.kweather.WeatherData GetForecast i 5

# 5. Monitor for updates
busctl --user monitor org.kde.kweather
```

### Unit Tests

Create tests for:
- D-Bus service registration
- Method return values
- Signal emission on data update
- Error handling (no location set, network error)

### Integration Testing

Test with Watchmate:
1. Install KDE Weather with D-Bus support
2. Run Watchmate
3. Verify KDE Weather appears in provider dropdown
4. Select KDE Weather as provider
5. Verify weather data appears on connected watch

## Privacy and Security

### Considerations

- Only exposes data already visible in the UI
- No network requests on D-Bus calls (use cached data)
- No control methods (read-only interface)
- Location is user-configured (not GPS tracking)

### User Control

- D-Bus service active only while KWeather is running
- Users can disable D-Bus interface via settings if desired
- Standard D-Bus session bus security applies

## Documentation

Update KWeather documentation to include:
- D-Bus interface availability
- Example client code
- Integration guide for third-party apps
- Reference to full D-Bus specification

## Backwards Compatibility

- This is a pure addition - no existing code changes
- D-Bus interface is optional - KWeather works without it
- No configuration file format changes
- No UI changes required

## Future Enhancements

Potential follow-ups (not in initial PR):
- Weather alerts via D-Bus
- Hourly forecast in addition to daily
- Multi-location support
- D-Bus activation (start KWeather via D-Bus call)

## Pull Request Checklist

- [ ] Implement D-Bus interface classes
- [ ] Integrate with existing weather model
- [ ] Add D-Bus service registration to main app
- [ ] Implement weather code mapping
- [ ] Add unit tests
- [ ] Update documentation
- [ ] Test with real D-Bus clients
- [ ] Add CMake/build configuration
- [ ] Handle edge cases (no location, no data)
- [ ] Emit signals on data updates

## Example PR Description

```markdown
### Add D-Bus Interface for Weather Data Access

This PR adds a D-Bus interface to expose KDE Weather's weather data to other applications on the system.

**Motivation**
Enable integration with smartwatch companion apps (like Watchmate for InfiniTime/PineTime) and other desktop utilities that need weather information.

**Changes**
- Add `org.kde.kweather.WeatherData` D-Bus interface
- Expose current weather via `GetCurrentWeather()` method
- Expose forecast via `GetForecast(days)` method
- Emit `WeatherUpdated` signal on data refresh
- No changes to existing functionality

**Testing**
Tested with busctl and Watchmate companion app. Weather data successfully retrieved and sent to smartwatch.

**Documentation**
Updated with D-Bus interface documentation and integration examples.

Closes: [Issue #XXX - Add D-Bus interface for external integration]
```

## Contact and Discussion

For questions or feedback on this proposal:
- KDE Weather issue tracker
- Plasma-devel mailing list
- IRC: #kde-devel

## References

- [D-Bus Interface Specification](DBUS_INTERFACE_SPEC.md)
- [Qt D-Bus Documentation](https://doc.qt.io/qt-6/qtdbus-index.html)
- [Watchmate Weather Provider Implementation](../WEATHER_IMPLEMENTATION.md)
- [MPRIS D-Bus Specification](https://specifications.freedesktop.org/mpris-spec/latest/) (similar pattern)
