# GNOME Weather D-Bus Interface - Pull Request Proposal

## Overview

This document describes the proposed changes to GNOME Weather to expose weather data via D-Bus, enabling integration with smartwatch companion apps and other desktop utilities.

## Motivation

Currently, GNOME Weather stores and displays weather data but doesn't expose it via D-Bus for other applications to consume. This limits integration opportunities:

- **Smartwatch Integration**: Companion apps like Watchmate need weather data to send to smartwatches (InfiniTime on PineTime)
- **GNOME Shell Extensions**: Extensions could display weather without implementing their own backend
- **Automation**: Scripts and smart home systems could react to weather conditions
- **Third-party Apps**: Other applications could provide alternative weather interfaces

## Benefits

1. **Ecosystem Integration**: Enables GNOME Weather to be a weather data provider for the broader Linux desktop ecosystem
2. **Smartwatch Support**: Allows GNOME users to see weather on their smartwatches via companion apps
3. **No User Impact**: This is a pure addition - no existing functionality changes
4. **Privacy Preserved**: Only exposes what's already visible in the UI
5. **GNOME Best Practices**: Follows D-Bus patterns used by other GNOME apps

## Use Cases

### Primary: Smartwatch Integration

Watchmate (companion app for InfiniTime/PineTime smartwatch) would:
1. Detect GNOME Weather via D-Bus service name
2. Fetch current weather and forecast
3. Send to connected smartwatch
4. Subscribe to updates for automatic refresh

### Secondary: Desktop Integration

- GNOME Shell extensions showing weather without separate backend
- Shell scripts checking weather for automation
- Notification systems for weather alerts

## Technical Design

### D-Bus Interface

**Service Name**: `org.gnome.Weather`  
**Object Path**: `/org/gnome/Weather`  
**Interface Name**: `org.gnome.Weather.WeatherData`

### Methods

#### GetCurrentWeather

Returns current weather conditions as a dictionary.

**Signature**: `GetCurrentWeather() → a{sv}`

**Returns**: GVariant dictionary with:
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

Returns weather forecast for upcoming days.

**Signature**: `GetForecast(i days) → aa{sv}`

**Parameters**:
- `days` (int32): Number of forecast days requested

**Returns**: Array of GVariant dictionaries, one per forecast day.

#### GetLocation

Returns configured location name.

**Signature**: `GetLocation() → s`

**Returns**: Location name string

### Properties

- `Location` (string, read-only): Current location
- `LastUpdate` (int64, read-only): Last update timestamp
- `IsValid` (boolean, read-only): Whether data is valid

### Signals

- `WeatherUpdated()`: Emitted when weather data is refreshed

## Implementation Approach

### File Structure

```
src/
├── weatherDBusService.js    # D-Bus service implementation
├── weatherInterface.xml     # D-Bus interface definition (introspection)
└── main.js                  # Integration point
```

### Integration Points

1. **Application Startup**: Register D-Bus service when app starts
2. **World Model**: Connect to existing GWeatherInfo data
3. **Update Callbacks**: Emit signal when weather updates
4. **Shutdown**: Cleanup D-Bus service

### Code Sketch (GJS/JavaScript)

```javascript
// weatherDBusService.js
const Gio = imports.gi.Gio;
const GLib = imports.gi.GLib;

const WeatherDataInterface = `
<node>
  <interface name="org.gnome.Weather.WeatherData">
    <method name="GetCurrentWeather">
      <arg type="a{sv}" direction="out" name="weather"/>
    </method>
    <method name="GetForecast">
      <arg type="i" direction="in" name="days"/>
      <arg type="aa{sv}" direction="out" name="forecast"/>
    </method>
    <method name="GetLocation">
      <arg type="s" direction="out" name="location"/>
    </method>
    
    <property name="Location" type="s" access="read"/>
    <property name="LastUpdate" type="x" access="read"/>
    <property name="IsValid" type="b" access="read"/>
    
    <signal name="WeatherUpdated"/>
  </interface>
</node>`;

var WeatherDBusService = class {
    constructor(app) {
        this._app = app;
        this._world = app.world;
        
        this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(
            WeatherDataInterface, this);
        
        this._dbusImpl.export(
            Gio.DBus.session,
            '/org/gnome/Weather');
        
        Gio.DBus.session.own_name(
            'org.gnome.Weather',
            Gio.BusNameOwnerFlags.NONE,
            null, null);
        
        // Connect to weather updates
        this._updateId = this._world.connect('updated', () => {
            this._dbusImpl.emit_signal('WeatherUpdated', null);
        });
    }
    
    GetCurrentWeather() {
        let info = this._getCurrentInfo();
        if (!info || !info.is_valid())
            return {};
        
        let weather = {
            'timestamp': GLib.Variant.new_int64(GLib.DateTime.new_now_local().to_unix()),
            'temperature': GLib.Variant.new_double(info.get_temp()),
            'temperatureMin': GLib.Variant.new_double(info.get_temp_min()),
            'temperatureMax': GLib.Variant.new_double(info.get_temp_max()),
            'location': GLib.Variant.new_string(this._world.getLocation().get_name()),
            'weatherCode': GLib.Variant.new_string(this._mapWeatherCode(info)),
            'weatherDescription': GLib.Variant.new_string(info.get_weather_summary()),
        };
        
        // Add sunrise/sunset if available
        let [hasSunrise, sunrise] = info.get_value_sunrise();
        if (hasSunrise) {
            weather['sunrise'] = GLib.Variant.new_int64(sunrise);
        }
        
        let [hasSunset, sunset] = info.get_value_sunset();
        if (hasSunset) {
            weather['sunset'] = GLib.Variant.new_int64(sunset);
        }
        
        return weather;
    }
    
    GetForecast(days) {
        let forecast = [];
        let infos = this._world.get_forecast();
        
        if (!infos)
            return forecast;
        
        let maxDays = Math.min(days, infos.length);
        
        for (let i = 0; i < maxDays; i++) {
            let info = infos[i];
            if (!info.is_valid())
                continue;
            
            let day = {
                'date': GLib.Variant.new_string(
                    GLib.DateTime.new_from_unix_local(info.get_update()).format('%Y-%m-%d')),
                'timestamp': GLib.Variant.new_int64(info.get_update()),
                'temperatureMin': GLib.Variant.new_double(info.get_temp_min()),
                'temperatureMax': GLib.Variant.new_double(info.get_temp_max()),
                'weatherCode': GLib.Variant.new_string(this._mapWeatherCode(info)),
                'weatherDescription': GLib.Variant.new_string(info.get_weather_summary()),
            };
            
            forecast.push(day);
        }
        
        return forecast;
    }
    
    GetLocation() {
        return this._world.getLocation().get_name();
    }
    
    get Location() {
        return this.GetLocation();
    }
    
    get LastUpdate() {
        let info = this._getCurrentInfo();
        if (!info || !info.is_valid())
            return 0;
        return info.get_update();
    }
    
    get IsValid() {
        let info = this._getCurrentInfo();
        return info && info.is_valid();
    }
    
    _getCurrentInfo() {
        return this._world.get_current_info();
    }
    
    _mapWeatherCode(info) {
        // Map GWeather conditions to standard codes
        let condition = info.get_condition();
        
        const codeMap = {
            'clear': 'clear-sky',
            'few-clouds': 'partly-cloudy',
            'partly-cloudy': 'partly-cloudy',
            'overcast': 'cloudy',
            'fog': 'fog',
            'drizzle': 'light-rain',
            'rain': 'rain',
            'snow': 'snow',
            'storm': 'thunderstorm',
            'thunderstorm': 'thunderstorm',
        };
        
        return codeMap[condition] || 'unknown';
    }
    
    destroy() {
        if (this._updateId) {
            this._world.disconnect(this._updateId);
            this._updateId = 0;
        }
        
        this._dbusImpl.unexport();
    }
};
```

```javascript
// In main.js Application class:

vfunc_startup() {
    super.vfunc_startup();
    
    // ... existing startup code ...
    
    // Register D-Bus service
    try {
        this._dbusService = new WeatherDBusService(this);
        log('D-Bus interface registered successfully');
    } catch (e) {
        logError(e, 'Failed to register D-Bus interface');
    }
}

vfunc_shutdown() {
    if (this._dbusService) {
        this._dbusService.destroy();
        this._dbusService = null;
    }
    
    super.vfunc_shutdown();
}
```

## Weather Code Mapping

Map GWeather condition codes to standard D-Bus codes:

```javascript
function mapWeatherCode(info) {
    // GWeather provides condition strings
    // Map to standard weather codes
    const condition = info.get_condition();
    
    const codeMap = {
        'clear': 'clear-sky',
        'few-clouds': 'partly-cloudy',
        'partly-cloudy': 'partly-cloudy',
        'overcast': 'cloudy',
        'fog': 'fog',
        'mist': 'fog',
        'drizzle': 'light-rain',
        'rain': 'rain',
        'snow': 'snow',
        'sleet': 'snow',
        'storm': 'thunderstorm',
        'thunderstorm': 'thunderstorm',
    };
    
    return codeMap[condition] || 'unknown';
}
```

## Testing

### Manual Testing

```bash
# 1. Build and run GNOME Weather with D-Bus support

# 2. Check service is registered
busctl --user list | grep gnome.Weather

# 3. Get current weather
busctl --user call org.gnome.Weather /org/gnome/Weather \
  org.gnome.Weather.WeatherData GetCurrentWeather

# 4. Get forecast
busctl --user call org.gnome.Weather /org/gnome/Weather \
  org.gnome.Weather.WeatherData GetForecast i 5

# 5. Monitor for updates
busctl --user monitor org.gnome.Weather
```

### Unit Tests

Create tests for:
- D-Bus service registration
- Method return values
- Signal emission on data update
- Error handling (no location set, network error)
- Weather code mapping

### Integration Testing

Test with Watchmate:
1. Install GNOME Weather with D-Bus support
2. Run Watchmate
3. Verify GNOME Weather appears in provider dropdown
4. Select GNOME Weather as provider
5. Verify weather data appears on connected watch

## Privacy and Security

### Considerations

- Only exposes data already visible in the UI
- No network requests on D-Bus calls (use cached data)
- No control methods (read-only interface)
- Location is user-configured (not GPS tracking)

### User Control

- D-Bus service active only while GNOME Weather is running
- Standard D-Bus session bus security applies
- No additional permissions required

## Documentation

Update GNOME Weather documentation to include:
- D-Bus interface availability
- Example client code
- Integration guide for third-party apps
- Reference to full D-Bus specification

## Backwards Compatibility

- This is a pure addition - no existing code changes
- D-Bus interface is optional - GNOME Weather works without it
- No GSettings schema changes
- No UI changes required

## Future Enhancements

Potential follow-ups (not in initial PR):
- Weather alerts via D-Bus
- Hourly forecast in addition to daily
- Multi-location support
- D-Bus activation (start GNOME Weather via D-Bus call)

## Pull Request Checklist

- [ ] Implement D-Bus service in GJS
- [ ] Integrate with existing GWeatherInfo
- [ ] Add D-Bus service registration to main app
- [ ] Implement weather code mapping
- [ ] Add tests
- [ ] Update documentation
- [ ] Test with real D-Bus clients
- [ ] Add to meson build
- [ ] Handle edge cases (no location, no data)
- [ ] Emit signals on data updates

## Example PR Description

```markdown
### Add D-Bus Interface for Weather Data Access

This PR adds a D-Bus interface to expose GNOME Weather's weather data to other applications on the system.

**Motivation**
Enable integration with smartwatch companion apps (like Watchmate for InfiniTime/PineTime) and other desktop utilities that need weather information.

**Changes**
- Add `org.gnome.Weather.WeatherData` D-Bus interface
- Expose current weather via `GetCurrentWeather()` method
- Expose forecast via `GetForecast(days)` method
- Emit `WeatherUpdated` signal on data refresh
- No changes to existing functionality

**Implementation**
- New `weatherDBusService.js` module
- Integrates with existing World/GWeatherInfo
- Automatic signal emission on weather updates

**Testing**
Tested with busctl and Watchmate companion app. Weather data successfully retrieved and sent to smartwatch.

**Documentation**
Added D-Bus interface documentation and integration examples.

Closes: #XXX - Add D-Bus interface for external integration
```

## Contact and Discussion

For questions or feedback on this proposal:
- GNOME Weather issue tracker (GitLab)
- GNOME Apps mailing list
- IRC/Matrix: #gnome-apps

## References

- [D-Bus Interface Specification](DBUS_INTERFACE_SPEC.md)
- [GDBus Documentation](https://docs.gtk.org/gio/class.DBusConnection.html)
- [GJS D-Bus Examples](https://gjs-docs.gnome.org/)
- [Watchmate Weather Provider Implementation](../WEATHER_IMPLEMENTATION.md)
- [GNOME Weather Source](https://gitlab.gnome.org/GNOME/gnome-weather)
