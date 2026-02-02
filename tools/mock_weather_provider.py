#!/usr/bin/env python3
"""
Mock Weather Provider for testing Watchmate weather integration.

This script creates a D-Bus service that simulates a weather application,
allowing you to test the weather provider integration without requiring
actual weather apps to be installed.

Usage:
    python3 mock_weather_provider.py [--provider-name NAME]

The mock provider will expose weather data via D-Bus that Watchmate can consume.
"""

import argparse
import random
from datetime import datetime
from gi.repository import GLib
from pydbus import SessionBus
from pydbus.generic import signal

# Mock weather data
CITIES = ["New York", "London", "Tokyo", "Paris", "Sydney"]
WEATHER_CONDITIONS = [
    ("clear-sky", "Clear sky", 0),
    ("few-clouds", "Few clouds", 5),
    ("scattered-clouds", "Cloudy", 10),
    ("rain", "Rain", -5),
    ("snow", "Snow", -10),
    ("thunderstorm", "Thunderstorm", 0),
]


class MockWeatherProvider:
    """
    Mock weather provider implementing the org.freedesktop.Weather D-Bus interface.
    """
    
    dbus = """
    <node>
      <interface name='org.freedesktop.Weather'>
        <method name='GetCurrentWeather'>
          <arg type='a{sv}' name='weather' direction='out'/>
        </method>
        <method name='GetForecast'>
          <arg type='a{sv}' name='forecast' direction='out'/>
        </method>
        <signal name='WeatherUpdated'>
          <arg type='x' name='timestamp'/>
        </signal>
      </interface>
    </node>
    """
    
    WeatherUpdated = signal()
    
    def __init__(self, location=None):
        self.location = location or random.choice(CITIES)
        self.base_temp = random.uniform(15.0, 25.0)
        self.current_condition = random.choice(WEATHER_CONDITIONS)
        print(f"Mock Weather Provider initialized for {self.location}")
        print(f"Base temperature: {self.base_temp:.1f}°C")
        print(f"Weather condition: {self.current_condition[1]}")
    
    def GetCurrentWeather(self):
        """Return current weather data."""
        condition_code, _, temp_offset = self.current_condition
        temp = self.base_temp + temp_offset + random.uniform(-2.0, 2.0)
        min_temp = temp - random.uniform(2.0, 5.0)
        max_temp = temp + random.uniform(2.0, 5.0)
        
        # Sunrise and sunset in minutes since midnight
        sunrise = 6 * 60 + random.randint(0, 60)  # Around 6 AM
        sunset = 18 * 60 + random.randint(0, 60)  # Around 6 PM
        
        weather = {
            'location': GLib.Variant('s', self.location),
            'temperature': GLib.Variant('d', temp),
            'min_temperature': GLib.Variant('d', min_temp),
            'max_temperature': GLib.Variant('d', max_temp),
            'icon_code': GLib.Variant('s', condition_code),
            'timestamp': GLib.Variant('x', int(datetime.now().timestamp())),
            'sunrise': GLib.Variant('q', sunrise),
            'sunset': GLib.Variant('q', sunset),
        }
        
        print(f"GetCurrentWeather called: {temp:.1f}°C, {condition_code}")
        return weather
    
    def GetForecast(self):
        """Return 5-day forecast data."""
        days = []
        for i in range(5):
            condition_code, _, temp_offset = random.choice(WEATHER_CONDITIONS)
            day_base = self.base_temp + random.uniform(-5.0, 5.0)
            min_temp = day_base - random.uniform(2.0, 5.0)
            max_temp = day_base + random.uniform(2.0, 5.0)
            
            days.append({
                'min_temperature': GLib.Variant('d', min_temp),
                'max_temperature': GLib.Variant('d', max_temp),
                'icon_code': GLib.Variant('s', condition_code),
            })
        
        forecast = {
            'timestamp': GLib.Variant('x', int(datetime.now().timestamp())),
            'days': GLib.Variant('aa{sv}', days),
        }
        
        print(f"GetForecast called: {len(days)} days")
        return forecast
    
    def update_weather(self):
        """Simulate weather update (for testing signals)."""
        self.current_condition = random.choice(WEATHER_CONDITIONS)
        self.WeatherUpdated(int(datetime.now().timestamp()))
        print(f"Weather updated: {self.current_condition[1]}")
        return True  # Continue the timer


def main():
    parser = argparse.ArgumentParser(description='Mock Weather Provider for Watchmate testing')
    parser.add_argument('--provider-name', default='MockWeather',
                       help='D-Bus service name suffix (default: MockWeather)')
    parser.add_argument('--location', default=None,
                       help='Location name (default: random city)')
    parser.add_argument('--update-interval', type=int, default=300,
                       help='Weather update interval in seconds (default: 300)')
    args = parser.parse_args()
    
    bus = SessionBus()
    service_name = f'org.example.{args.provider_name}'
    
    try:
        provider = MockWeatherProvider(location=args.location)
        bus.publish(service_name, ('/', provider))
        print(f"\nMock Weather Provider registered on D-Bus as '{service_name}'")
        print("You can now select it in Watchmate's weather provider dropdown.")
        print("\nTest with:")
        print(f"  busctl --user call {service_name} / org.freedesktop.Weather GetCurrentWeather")
        print(f"\nPress Ctrl+C to stop.\n")
        
        # Set up periodic weather updates
        if args.update_interval > 0:
            GLib.timeout_add_seconds(args.update_interval, provider.update_weather)
            print(f"Weather will update every {args.update_interval} seconds\n")
        
        loop = GLib.MainLoop()
        loop.run()
    except KeyboardInterrupt:
        print("\nMock Weather Provider stopped.")
    except Exception as e:
        print(f"Error: {e}")
        return 1
    
    return 0


if __name__ == '__main__':
    exit(main())
