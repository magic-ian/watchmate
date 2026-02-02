use super::super::uuids;
use super::InfiniTime;
use anyhow::Result;

/// Maximum number of forecast days supported by InfiniTime SimpleWeatherService
const MAX_FORECAST_DAYS: usize = 5;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum WeatherIcon {
    Sun = 0,
    FewClouds = 1,
    Clouds = 2,
    HeavyClouds = 3,
    CloudsRain = 4,
    Rain = 5,
    Thunderstorm = 6,
    Snow = 7,
    Mist = 8,
}

#[derive(Debug, Clone)]
pub struct CurrentWeather {
    pub timestamp: i64,
    pub temperature: i16,
    pub min_temperature: i16,
    pub max_temperature: i16,
    pub location: String,
    pub icon: WeatherIcon,
    pub sunrise: Option<i16>,
    pub sunset: Option<i16>,
}

#[derive(Debug, Clone)]
pub struct DayForecast {
    pub min_temperature: i16,
    pub max_temperature: i16,
    pub icon: WeatherIcon,
}

#[derive(Debug, Clone)]
pub struct WeatherForecast {
    pub timestamp: i64,
    pub days: Vec<DayForecast>,
}

impl InfiniTime {
    pub async fn send_current_weather(&self, weather: &CurrentWeather) -> Result<()> {
        let mut data = Vec::with_capacity(53);
        
        // Message type and version
        data.push(0); // Current weather
        data.push(1); // Version with sunrise/sunset support
        
        // Timestamp (8 bytes, little-endian)
        data.extend_from_slice(&weather.timestamp.to_le_bytes());
        
        // Temperatures (°C * 100, 2 bytes each, little-endian)
        data.extend_from_slice(&weather.temperature.to_le_bytes());
        data.extend_from_slice(&weather.min_temperature.to_le_bytes());
        data.extend_from_slice(&weather.max_temperature.to_le_bytes());
        
        // Location (32 bytes, null-terminated string)
        let mut location_bytes = [0u8; 32];
        let location_str = weather.location.as_bytes();
        let copy_len = location_str.len().min(31); // Leave room for null terminator
        location_bytes[..copy_len].copy_from_slice(&location_str[..copy_len]);
        data.extend_from_slice(&location_bytes);
        
        // Icon ID
        data.push(weather.icon as u8);
        
        // Sunrise (2 bytes, little-endian)
        let sunrise = weather.sunrise.unwrap_or(-1);
        data.extend_from_slice(&sunrise.to_le_bytes());
        
        // Sunset (2 bytes, little-endian)
        let sunset = weather.sunset.unwrap_or(-1);
        data.extend_from_slice(&sunset.to_le_bytes());
        
        self.chr(&uuids::CHR_WEATHER_DATA)?.write(&data).await?;
        Ok(())
    }
    
    pub async fn send_weather_forecast(&self, forecast: &WeatherForecast) -> Result<()> {
        let mut data = Vec::with_capacity(36);
        
        // Message type and version
        data.push(1); // Forecast
        data.push(0); // Version 0
        
        // Timestamp (8 bytes, little-endian)
        data.extend_from_slice(&forecast.timestamp.to_le_bytes());
        
        // Number of days (max MAX_FORECAST_DAYS)
        let num_days = forecast.days.len().min(MAX_FORECAST_DAYS) as u8;
        data.push(num_days);
        
        // Day forecasts (MAX_FORECAST_DAYS days, 5 bytes each)
        for day in &forecast.days[..num_days as usize] {
            data.extend_from_slice(&day.min_temperature.to_le_bytes());
            data.extend_from_slice(&day.max_temperature.to_le_bytes());
            data.push(day.icon as u8);
        }
        
        // Pad remaining days with zeros if needed
        for _ in num_days..(MAX_FORECAST_DAYS as u8) {
            data.extend_from_slice(&[0, 0, 0, 0, 0]);
        }
        
        self.chr(&uuids::CHR_WEATHER_DATA)?.write(&data).await?;
        Ok(())
    }
}
