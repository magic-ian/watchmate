// Example usage of the weather service API
// This file demonstrates how to use the weather provider infrastructure

use infinitime::bt::device::weather::{CurrentWeather, WeatherIcon, WeatherForecast, DayForecast};
use infinitime::bt::InfiniTime;
use std::sync::Arc;

/// Example: Send current weather to the watch
pub async fn example_send_current_weather(device: &Arc<InfiniTime>) -> anyhow::Result<()> {
    let weather = CurrentWeather {
        timestamp: chrono::Utc::now().timestamp(),
        temperature: 2250,  // 22.50°C
        min_temperature: 1800,  // 18.00°C
        max_temperature: 2600,  // 26.00°C
        location: "San Francisco".to_string(),
        icon: WeatherIcon::FewClouds,
        sunrise: Some(6 * 60 + 30),  // 6:30 AM
        sunset: Some(18 * 60 + 45),  // 6:45 PM
    };
    
    device.send_current_weather(&weather).await?;
    println!("✓ Sent current weather to watch");
    Ok(())
}

/// Example: Send 5-day forecast to the watch
pub async fn example_send_forecast(device: &Arc<InfiniTime>) -> anyhow::Result<()> {
    let forecast = WeatherForecast {
        timestamp: chrono::Utc::now().timestamp(),
        days: vec![
            DayForecast {
                min_temperature: 1800,  // 18°C
                max_temperature: 2600,  // 26°C
                icon: WeatherIcon::Sun,
            },
            DayForecast {
                min_temperature: 1900,  // 19°C
                max_temperature: 2700,  // 27°C
                icon: WeatherIcon::FewClouds,
            },
            DayForecast {
                min_temperature: 1700,  // 17°C
                max_temperature: 2300,  // 23°C
                icon: WeatherIcon::CloudsRain,
            },
            DayForecast {
                min_temperature: 1600,  // 16°C
                max_temperature: 2100,  // 21°C
                icon: WeatherIcon::Rain,
            },
            DayForecast {
                min_temperature: 1800,  // 18°C
                max_temperature: 2400,  // 24°C
                icon: WeatherIcon::Clouds,
            },
        ],
    };
    
    device.send_weather_forecast(&forecast).await?;
    println!("✓ Sent 5-day forecast to watch");
    Ok(())
}

/// Convert Fahrenheit to Celsius (×100 for protocol)
pub fn fahrenheit_to_celsius_x100(fahrenheit: f32) -> i16 {
    ((fahrenheit - 32.0) * 5.0 / 9.0 * 100.0) as i16
}

/// Convert Celsius to protocol format (×100)
pub fn celsius_to_protocol(celsius: f32) -> i16 {
    (celsius * 100.0) as i16
}

/// Convert time (hours, minutes) to minutes since midnight
pub fn time_to_minutes(hours: u8, minutes: u8) -> i16 {
    (hours as i16 * 60) + (minutes as i16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_conversion() {
        assert_eq!(celsius_to_protocol(22.5), 2250);
        assert_eq!(celsius_to_protocol(-5.0), -500);
        assert_eq!(fahrenheit_to_celsius_x100(72.0), 2222);
    }

    #[test]
    fn test_time_conversion() {
        assert_eq!(time_to_minutes(6, 30), 390);
        assert_eq!(time_to_minutes(18, 45), 1125);
        assert_eq!(time_to_minutes(0, 0), 0);
        assert_eq!(time_to_minutes(23, 59), 1439);
    }
}
