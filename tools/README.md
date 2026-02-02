# Watchmate Development Tools

This directory contains tools and utilities for developing and testing Watchmate features.

## Mock Weather Provider

`mock_weather_provider.py` - A D-Bus service that simulates a weather application for testing the weather provider integration.

### Requirements

```bash
# Install dependencies
pip install pydbus PyGObject
```

### Usage

```bash
# Run with default settings
./mock_weather_provider.py

# Run with custom provider name
./mock_weather_provider.py --provider-name MyWeather

# Run with specific location
./mock_weather_provider.py --location "San Francisco"

# Run with custom update interval (in seconds)
./mock_weather_provider.py --update-interval 60
```

### Testing

Once the mock provider is running, you can test it with:

```bash
# Check if service is registered
busctl --user list | grep org.example

# Call GetCurrentWeather method
busctl --user call org.example.MockWeather / org.freedesktop.Weather GetCurrentWeather

# Call GetForecast method
busctl --user call org.example.MockWeather / org.freedesktop.Weather GetForecast

# Monitor D-Bus signals
busctl --user monitor org.example.MockWeather
```

### Integration with Watchmate

1. Start the mock weather provider
2. Launch Watchmate and connect to your InfiniTime watch
3. The mock provider should appear in the weather provider dropdown
4. Select it to start receiving weather updates on your watch

Note: The actual weather data transmission to the watch will only work once the full integration is complete.
