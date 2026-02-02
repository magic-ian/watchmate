use anyhow::{anyhow, Result};
use zbus::{Connection, proxy};
use std::sync::Arc;
use futures::{Stream, StreamExt};

use crate::bt::device::weather::{CurrentWeather, WeatherForecast, WeatherIcon};

/// D-Bus service name for KDE Weather
const KWEATHER_SERVICE: &str = "org.kde.kweather";

/// D-Bus service name for GNOME Weather
const GNOME_WEATHER_SERVICE: &str = "org.gnome.Weather";

#[derive(Debug, Clone)]
pub struct WeatherProvider {
    pub name: String,
    pub service_name: String,
}

impl WeatherProvider {
    pub fn new(name: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            service_name: service_name.into(),
        }
    }
}

/// Get list of available weather providers
pub async fn get_providers(connection: &Connection) -> Result<Vec<WeatherProvider>> {
    let mut providers = Vec::new();
    
    // Check for KDE Weather (org.kde.kweather)
    if check_service_exists(connection, KWEATHER_SERVICE).await? {
        providers.push(WeatherProvider::new("KDE Weather", KWEATHER_SERVICE));
    }
    
    // Check for GNOME Weather (org.gnome.Weather)
    if check_service_exists(connection, GNOME_WEATHER_SERVICE).await? {
        providers.push(WeatherProvider::new("GNOME Weather", GNOME_WEATHER_SERVICE));
    }
    
    Ok(providers)
}

/// Check if a D-Bus service exists
async fn check_service_exists(connection: &Connection, service_name: &str) -> Result<bool> {
    let dbus_proxy = zbus::fdo::DBusProxy::new(connection).await?;
    match dbus_proxy.name_has_owner(service_name.try_into()?).await {
        Ok(has_owner) => Ok(has_owner),
        Err(_) => Ok(false),
    }
}

/// Get current weather data from a provider
pub async fn get_current_weather(
    _connection: &Connection,
    provider: &WeatherProvider,
) -> Result<CurrentWeather> {
    // This is a placeholder implementation
    // Real implementation would query the D-Bus service
    Err(anyhow!(
        "Weather data retrieval not yet implemented for {}",
        provider.name
    ))
}

/// Get weather forecast from a provider
pub async fn get_forecast(
    _connection: &Connection,
    provider: &WeatherProvider,
) -> Result<WeatherForecast> {
    // This is a placeholder implementation
    // Real implementation would query the D-Bus service
    Err(anyhow!(
        "Weather forecast retrieval not yet implemented for {}",
        provider.name
    ))
}

/// Stream of weather provider updates
pub async fn get_providers_update_stream(
    connection: &Arc<Connection>,
) -> Result<impl Stream<Item = ProvidersListEvent>> {
    let dbus_proxy = zbus::fdo::DBusProxy::new(connection).await?;
    let stream = dbus_proxy
        .receive_name_owner_changed()
        .await?
        .filter_map(move |signal| {
            let args = signal.args().ok()?;
            let name = args.name.as_str();
            
            // Only monitor weather-related services
            let provider_info = match name {
                KWEATHER_SERVICE => Some("KDE Weather"),
                GNOME_WEATHER_SERVICE => Some("GNOME Weather"),
                _ => None,
            };
            
            provider_info.and_then(|provider_name| {
                if args.new_owner.is_some() {
                    Some(ProvidersListEvent::ProviderAdded(WeatherProvider::new(
                        provider_name,
                        name,
                    )))
                } else {
                    Some(ProvidersListEvent::ProviderRemoved(
                        args.name.clone().into_inner(),
                    ))
                }
            })
        });
    
    Ok(stream)
}

#[derive(Debug)]
pub enum ProvidersListEvent {
    ProviderAdded(WeatherProvider),
    ProviderRemoved(String),
}
