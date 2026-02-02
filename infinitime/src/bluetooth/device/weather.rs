use super::{uuids, InfiniTime};
use anyhow::Result;
use chrono::{DateTime, Local};

// According to InfiniTime protocol, -1 represents unknown sunrise/sunset time
const UNKNOWN_TIME: i16 = -1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WeatherIcon {
    Sun = 0,              // Clear sky
    CloudsSun = 1,        // Few clouds
    Clouds = 2,           // Scattered clouds
    BrokenClouds = 3,     // Heavy clouds
    CloudShowerHeavy = 4, // Shower rain
    CloudSunRain = 5,     // Rain
    Thunderstorm = 6,
    Snow = 7,
    Smog = 8, // Mist
}

#[derive(Debug, Clone)]
pub struct CurrentWeather {
    pub timestamp: DateTime<Local>,
    pub temperature: f32,        // in °C
    pub min_temperature: f32,    // in °C
    pub max_temperature: f32,    // in °C
    pub location: String,        // max 32 chars
    pub icon: WeatherIcon,
    pub sunrise: Option<u16>,    // minutes since midnight, None if unknown
    pub sunset: Option<u16>,     // minutes since midnight, None if unknown
}

#[derive(Debug, Clone)]
pub struct ForecastDay {
    pub min_temperature: f32, // in °C
    pub max_temperature: f32, // in °C
    pub icon: WeatherIcon,
}

#[derive(Debug, Clone)]
pub struct Forecast {
    pub timestamp: DateTime<Local>,
    pub days: Vec<ForecastDay>, // max 5 days
}

impl InfiniTime {
    /// Write current weather data to the watch
    pub async fn write_current_weather(&self, weather: &CurrentWeather) -> Result<()> {
        let mut data = Vec::with_capacity(53);

        // Message type and version
        data.push(0u8); // Current weather
        data.push(1u8); // Version 1 (with sunrise/sunset)

        // Timestamp (8 bytes, little-endian)
        let timestamp = weather.timestamp.timestamp() as u64;
        data.extend_from_slice(&timestamp.to_le_bytes());

        // Temperatures (in °C * 100, 2 bytes each, little-endian)
        let temp = (weather.temperature * 100.0) as i16;
        let min_temp = (weather.min_temperature * 100.0) as i16;
        let max_temp = (weather.max_temperature * 100.0) as i16;
        data.extend_from_slice(&temp.to_le_bytes());
        data.extend_from_slice(&min_temp.to_le_bytes());
        data.extend_from_slice(&max_temp.to_le_bytes());

        // Location (32 bytes, null-terminated)
        let location = weather.location.chars().take(32).collect::<String>();
        let mut location_bytes = location.into_bytes();
        location_bytes.resize(32, 0);
        data.extend_from_slice(&location_bytes);

        // Icon ID
        data.push(weather.icon as u8);

        // Sunrise and sunset (2 bytes each, little-endian)
        // Use UNKNOWN_TIME (-1) when sunrise/sunset data is not available
        let sunrise = weather.sunrise.map(|s| s as i16).unwrap_or(UNKNOWN_TIME);
        let sunset = weather.sunset.map(|s| s as i16).unwrap_or(UNKNOWN_TIME);
        data.extend_from_slice(&sunrise.to_le_bytes());
        data.extend_from_slice(&sunset.to_le_bytes());

        Ok(self.chr(&uuids::CHR_WEATHER_DATA)?.write(&data).await?)
    }

    /// Write forecast data to the watch
    pub async fn write_forecast(&self, forecast: &Forecast) -> Result<()> {
        let mut data = Vec::with_capacity(36);

        // Message type and version
        data.push(1u8); // Forecast
        data.push(0u8); // Version 0

        // Timestamp (8 bytes, little-endian)
        let timestamp = forecast.timestamp.timestamp() as u64;
        data.extend_from_slice(&timestamp.to_le_bytes());

        // Number of days (max 5)
        let nb_days = forecast.days.len().min(5) as u8;
        data.push(nb_days);

        // Forecast days (5 bytes each: min_temp, max_temp, icon)
        for day in forecast.days.iter().take(5) {
            let min_temp = (day.min_temperature * 100.0) as i16;
            let max_temp = (day.max_temperature * 100.0) as i16;
            data.extend_from_slice(&min_temp.to_le_bytes());
            data.extend_from_slice(&max_temp.to_le_bytes());
            data.push(day.icon as u8);
        }

        // Pad remaining days with zeros if less than 5
        for _ in forecast.days.len()..5 {
            data.extend_from_slice(&[0u8; 5]);
        }

        Ok(self.chr(&uuids::CHR_WEATHER_DATA)?.write(&data).await?)
    }
}
