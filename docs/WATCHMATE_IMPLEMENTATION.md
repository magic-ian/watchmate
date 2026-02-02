# Watchmate Implementation Guide for Weather Provider D-Bus Integration

This document describes how to implement the actual weather data fetching in Watchmate once KDE Weather and/or GNOME Weather expose D-Bus interfaces.

## Overview

Once weather apps expose the proposed D-Bus interfaces, watchmate's stub functions in `infinitime/src/freedesktop/weather.rs` need to be implemented to actually fetch and parse weather data.

## Implementation Plan

### 1. Create D-Bus Proxy Traits

Add zbus proxy traits for the weather interface:

```rust
// infinitime/src/freedesktop/weather.rs

use zbus::proxy;
use std::collections::HashMap;

#[proxy(
    interface = "org.kde.kweather.WeatherData",
    default_path = "/org/kde/kweather"
)]
trait KWeatherData {
    /// Get current weather conditions
    fn get_current_weather(&self) -> zbus::Result<HashMap<String, zbus::zvariant::Value>>;
    
    /// Get weather forecast
    fn get_forecast(&self, days: i32) -> zbus::Result<Vec<HashMap<String, zbus::zvariant::Value>>>;
    
    /// Get configured location
    fn get_location(&self) -> zbus::Result<String>;
    
    /// Weather data updated signal
    #[zbus(signal)]
    fn weather_updated(&self) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.gnome.Weather.WeatherData",
    default_path = "/org/gnome/Weather"
)]
trait GnomeWeatherData {
    /// Get current weather conditions
    fn get_current_weather(&self) -> zbus::Result<HashMap<String, zbus::zvariant::Value>>;
    
    /// Get weather forecast
    fn get_forecast(&self, days: i32) -> zbus::Result<Vec<HashMap<String, zbus::zvariant::Value>>>;
    
    /// Get configured location
    fn get_location(&self) -> zbus::Result<String>;
    
    /// Weather data updated signal
    #[zbus(signal)]
    fn weather_updated(&self) -> zbus::Result<()>;
}
```

### 2. Implement Data Parsing

Create helper functions to parse D-Bus variant dictionaries into watchmate types:

```rust
use crate::bt::device::weather::{CurrentWeather, WeatherForecast, WeatherIcon, DayForecast};
use chrono::{DateTime, Utc, TimeZone};

/// Parse D-Bus variant dictionary into CurrentWeather
fn parse_current_weather(
    data: HashMap<String, zbus::zvariant::Value>,
) -> Result<CurrentWeather> {
    // Helper to extract values from variant dict
    fn get_i64(map: &HashMap<String, zbus::zvariant::Value>, key: &str) -> Option<i64> {
        map.get(key)?.downcast_ref::<i64>().copied()
    }
    
    fn get_f64(map: &HashMap<String, zbus::zvariant::Value>, key: &str) -> Option<f64> {
        map.get(key)?.downcast_ref::<f64>().copied()
    }
    
    fn get_string(map: &HashMap<String, zbus::zvariant::Value>, key: &str) -> Option<String> {
        map.get(key)?.downcast_ref::<str>().map(String::from)
    }
    
    // Extract timestamp
    let timestamp = get_i64(&data, "timestamp")
        .ok_or_else(|| anyhow!("Missing timestamp"))?;
    
    // Extract temperatures (convert from Celsius to protocol format: Celsius * 100)
    let temperature = get_f64(&data, "temperature")
        .ok_or_else(|| anyhow!("Missing temperature"))?;
    let temp_min = get_f64(&data, "temperatureMin").unwrap_or(temperature);
    let temp_max = get_f64(&data, "temperatureMax").unwrap_or(temperature);
    
    // Extract location (truncate to 32 chars for protocol)
    let mut location = get_string(&data, "location")
        .ok_or_else(|| anyhow!("Missing location"))?;
    if location.len() > 31 {
        location.truncate(31);
    }
    
    // Map weather code to icon
    let weather_code = get_string(&data, "weatherCode").unwrap_or_default();
    let icon = map_weather_code_to_icon(&weather_code);
    
    // Extract sunrise/sunset (convert Unix timestamp to minutes since midnight)
    let sunrise = get_i64(&data, "sunrise")
        .and_then(|ts| timestamp_to_minutes_since_midnight(ts));
    let sunset = get_i64(&data, "sunset")
        .and_then(|ts| timestamp_to_minutes_since_midnight(ts));
    
    Ok(CurrentWeather {
        timestamp,
        temperature: (temperature * 100.0) as i16,
        min_temperature: (temp_min * 100.0) as i16,
        max_temperature: (temp_max * 100.0) as i16,
        location,
        icon,
        sunrise,
        sunset,
    })
}

/// Parse D-Bus forecast array into WeatherForecast
fn parse_forecast(
    data: Vec<HashMap<String, zbus::zvariant::Value>>,
    timestamp: i64,
) -> Result<WeatherForecast> {
    let mut days = Vec::new();
    
    for day_data in data.into_iter().take(5) {
        let temp_min = day_data.get("temperatureMin")
            .and_then(|v| v.downcast_ref::<f64>())
            .copied()
            .unwrap_or(0.0);
        
        let temp_max = day_data.get("temperatureMax")
            .and_then(|v| v.downcast_ref::<f64>())
            .copied()
            .unwrap_or(0.0);
        
        let weather_code = day_data.get("weatherCode")
            .and_then(|v| v.downcast_ref::<str>())
            .unwrap_or("");
        
        let icon = map_weather_code_to_icon(weather_code);
        
        days.push(DayForecast {
            min_temperature: (temp_min * 100.0) as i16,
            max_temperature: (temp_max * 100.0) as i16,
            icon,
        });
    }
    
    Ok(WeatherForecast {
        timestamp,
        days,
    })
}

/// Map standard weather code to InfiniTime icon
fn map_weather_code_to_icon(code: &str) -> WeatherIcon {
    match code {
        "clear-sky" | "sunny" => WeatherIcon::Sun,
        "partly-cloudy" | "few-clouds" => WeatherIcon::FewClouds,
        "cloudy" | "overcast" => WeatherIcon::Clouds,
        "heavy-clouds" => WeatherIcon::HeavyClouds,
        "light-rain" | "drizzle" => WeatherIcon::CloudsRain,
        "rain" | "rainy" => WeatherIcon::Rain,
        "thunderstorm" | "storm" => WeatherIcon::Thunderstorm,
        "snow" | "snowy" => WeatherIcon::Snow,
        "fog" | "mist" => WeatherIcon::Mist,
        _ => WeatherIcon::Clouds, // Default fallback
    }
}

/// Convert Unix timestamp to minutes since midnight in local time
fn timestamp_to_minutes_since_midnight(timestamp: i64) -> Option<i16> {
    let dt = Utc.timestamp_opt(timestamp, 0).single()?;
    let local = dt.with_timezone(&chrono::Local);
    let minutes = (local.hour() * 60 + local.minute()) as i16;
    Some(minutes)
}
```

### 3. Implement Actual Data Fetching

Replace the stub functions with real implementations:

```rust
/// Get current weather data from a provider
pub async fn get_current_weather(
    connection: &Connection,
    provider: &WeatherProvider,
) -> Result<CurrentWeather> {
    match provider.service_name.as_str() {
        KWEATHER_SERVICE => {
            let proxy = KWeatherDataProxy::builder(connection)
                .destination(KWEATHER_SERVICE)?
                .build()
                .await?;
            
            let data = proxy.get_current_weather().await?;
            parse_current_weather(data)
        }
        GNOME_WEATHER_SERVICE => {
            let proxy = GnomeWeatherDataProxy::builder(connection)
                .destination(GNOME_WEATHER_SERVICE)?
                .build()
                .await?;
            
            let data = proxy.get_current_weather().await?;
            parse_current_weather(data)
        }
        _ => Err(anyhow!(
            "Unsupported weather provider: {}",
            provider.service_name
        )),
    }
}

/// Get weather forecast from a provider
pub async fn get_forecast(
    connection: &Connection,
    provider: &WeatherProvider,
) -> Result<WeatherForecast> {
    let timestamp = chrono::Utc::now().timestamp();
    
    match provider.service_name.as_str() {
        KWEATHER_SERVICE => {
            let proxy = KWeatherDataProxy::builder(connection)
                .destination(KWEATHER_SERVICE)?
                .build()
                .await?;
            
            let data = proxy.get_forecast(5).await?;
            parse_forecast(data, timestamp)
        }
        GNOME_WEATHER_SERVICE => {
            let proxy = GnomeWeatherDataProxy::builder(connection)
                .destination(GNOME_WEATHER_SERVICE)?
                .build()
                .await?;
            
            let data = proxy.get_forecast(5).await?;
            parse_forecast(data, timestamp)
        }
        _ => Err(anyhow!(
            "Unsupported weather provider: {}",
            provider.service_name
        )),
    }
}
```

### 4. Update Weather Session to Fetch and Send Data

Update the weather session in `watchmate/src/ui/dashboard_page/weather.rs`:

```rust
Input::WeatherSessionStart => {
    if let Some(infinitime) = self.infinitime.clone() {
        let index = self.dropdown.selected() as usize;
        if index < self.provider_handles.len() {
            // Stop current weather session
            self.stop_weather_task();
            // Start new weather session
            let provider = self.provider_handles[index].clone();
            let dbus_session = self.dbus_session.clone();
            let task_handle = relm4::spawn(async move {
                log::info!(
                    "Weather session started for provider: {}",
                    provider.name
                );
                
                // Periodic weather update loop
                loop {
                    if let Some(ref dbus) = dbus_session {
                        // Fetch and send weather data
                        match weather::get_current_weather(dbus, &provider).await {
                            Ok(current) => {
                                if let Err(e) = infinitime.send_current_weather(&current).await {
                                    log::error!("Failed to send current weather: {}", e);
                                } else {
                                    log::info!("Sent current weather to watch");
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to fetch current weather: {}", e);
                            }
                        }
                        
                        // Fetch and send forecast
                        match weather::get_forecast(dbus, &provider).await {
                            Ok(forecast) => {
                                if let Err(e) = infinitime.send_weather_forecast(&forecast).await {
                                    log::error!("Failed to send forecast: {}", e);
                                } else {
                                    log::info!("Sent forecast to watch");
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to fetch forecast: {}", e);
                            }
                        }
                    }
                    
                    // Wait 30 minutes before next update
                    tokio::time::sleep(tokio::time::Duration::from_secs(1800)).await;
                }
            });
            self.weather_task = Some(task_handle);
        }
    }
}
```

## Testing

### Unit Tests

Add tests for parsing functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::Value;
    
    #[test]
    fn test_weather_code_mapping() {
        assert_eq!(map_weather_code_to_icon("clear-sky"), WeatherIcon::Sun);
        assert_eq!(map_weather_code_to_icon("rain"), WeatherIcon::Rain);
        assert_eq!(map_weather_code_to_icon("snow"), WeatherIcon::Snow);
        assert_eq!(map_weather_code_to_icon("unknown"), WeatherIcon::Clouds);
    }
    
    #[test]
    fn test_parse_current_weather() {
        let mut data = HashMap::new();
        data.insert("timestamp".to_string(), Value::new(1706860800i64));
        data.insert("temperature".to_string(), Value::new(22.5f64));
        data.insert("temperatureMin".to_string(), Value::new(18.0f64));
        data.insert("temperatureMax".to_string(), Value::new(26.0f64));
        data.insert("location".to_string(), Value::new("San Francisco"));
        data.insert("weatherCode".to_string(), Value::new("partly-cloudy"));
        
        let weather = parse_current_weather(data).unwrap();
        assert_eq!(weather.temperature, 2250);
        assert_eq!(weather.min_temperature, 1800);
        assert_eq!(weather.max_temperature, 2600);
        assert_eq!(weather.location, "San Francisco");
    }
}
```

### Integration Testing

1. **Mock D-Bus Service**: Create a mock weather service for testing
2. **End-to-End Test**: Test complete flow from D-Bus fetch to watch send
3. **Error Handling**: Test behavior when weather app is not running

## Error Handling

Implement proper error handling:

```rust
// Handle provider not responding
match weather::get_current_weather(dbus, &provider).await {
    Ok(current) => { /* send to watch */ },
    Err(e) if e.to_string().contains("NameHasNoOwner") => {
        log::warn!("Weather provider stopped: {}", provider.name);
        // Could trigger ProviderRemoved event
    },
    Err(e) => {
        log::error!("Weather fetch error: {}", e);
        // Retry on next cycle
    },
}
```

## User Configuration

Future enhancement - add settings for:
- Update frequency (15min, 30min, 1hr, 2hr)
- Temperature units (auto-convert from Celsius if needed)
- Enable/disable automatic updates

## Performance Considerations

- Cache weather data locally
- Don't spam D-Bus with requests
- Use signals to detect updates rather than polling
- Implement exponential backoff on errors

## Documentation Updates

Update user documentation:
- Explain weather provider setup
- List supported weather apps
- Troubleshooting guide

## Deployment

1. Update watchmate dependencies if needed
2. Test with both KDE Weather and GNOME Weather
3. Add to release notes
4. Update README with weather feature

## References

- [D-Bus Interface Specification](DBUS_INTERFACE_SPEC.md)
- [KDE Weather PR Proposal](KDE_WEATHER_PR_PROPOSAL.md)
- [GNOME Weather PR Proposal](GNOME_WEATHER_PR_PROPOSAL.md)
- [zbus Documentation](https://docs.rs/zbus/)
