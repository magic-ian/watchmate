# Weather Provider Testing Guide

This guide explains how to test the weather provider selector implementation.

## Prerequisites

### Build Environment

1. Install GTK4 and Libadwaita development libraries:
   ```bash
   # Ubuntu/Debian
   sudo apt install libgtk-4-dev libadwaita-1-dev libdbus-1-dev pkg-config
   
   # Fedora
   sudo dnf install gtk4-devel libadwaita-devel dbus-devel pkgconf-pkg-config
   
   # Arch
   sudo pacman -S gtk4 libadwaita dbus
   ```

2. Install Rust (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Weather Applications

Install at least one weather provider:

#### KDE Weather
```bash
# Flatpak (recommended)
flatpak install org.kde.kweather

# Or from package manager
sudo apt install kweather  # Debian/Ubuntu
sudo dnf install kweather  # Fedora
```

#### GNOME Weather
```bash
# Flatpak (recommended)
flatpak install org.gnome.Weather

# Or from package manager
sudo apt install gnome-weather  # Debian/Ubuntu
sudo dnf install gnome-weather  # Fedora
```

### InfiniTime Watch

- PineTime watch with InfiniTime firmware 1.14.0 or later (for SimpleWeatherService support)
- Bluetooth enabled on your computer
- Watch paired and connected

## Building Watchmate

1. Clone the repository (if not already):
   ```bash
   git clone https://github.com/magic-ian/watchmate.git
   cd watchmate
   ```

2. Install GLib schema for settings:
   ```bash
   mkdir -p $HOME/.local/share/glib-2.0/schemas/
   cp assets/io.gitlab.azymohliad.WatchMate.gschema.xml $HOME/.local/share/glib-2.0/schemas/
   glib-compile-schemas $HOME/.local/share/glib-2.0/schemas/
   ```

3. Build and run:
   ```bash
   cargo run --release
   ```

## Testing Procedure

### 1. Test Provider Detection

**Without any weather app running:**
1. Launch Watchmate
2. Connect to your InfiniTime watch
3. Navigate to the Dashboard
4. Under "Host Integration", you should see:
   - Media Player: [media player selector]
   - Weather Provider: "Not available"
   - Notifications: [toggle]

**With weather app running:**
1. Start KDE Weather or GNOME Weather
2. The "Weather Provider" row should update to show a dropdown
3. The dropdown should list the running weather app(s)

**Test dynamic detection:**
1. Close the weather app while Watchmate is running
2. The dropdown should disappear and show "Not available"
3. Restart the weather app
4. The dropdown should reappear with the provider listed

### 2. Test Provider Selection

1. If multiple weather apps are running, both should appear in the dropdown
2. Select each provider and verify the selection works
3. Check the logs (run with `RUST_LOG=info cargo run --release`) for:
   - "Weather provider started: [provider name]"
   - "Weather session started for provider: [provider name]"

### 3. Test D-Bus Service Detection

Verify D-Bus service names:

```bash
# List all session bus names
dbus-send --session --dest=org.freedesktop.DBus --type=method_call --print-reply /org/freedesktop/DBus org.freedesktop.DBus.ListNames

# Should include:
# - org.kde.kweather (if KDE Weather is running)
# - org.gnome.Weather (if GNOME Weather is running)
```

### 4. Test with Different Installation Methods

Test each combination:

1. **Native packages:**
   - Install via system package manager
   - Verify detection works

2. **Flatpak:**
   - Install via Flatpak
   - Verify D-Bus service name is correct
   - If service name differs (e.g., org.flatpak.kweather), update the code in:
     - `infinitime/src/freedesktop/weather.rs` (service names)
     - `watchmate/src/ui/dashboard_page/weather.rs` (if needed)

3. **Multiple installations:**
   - Have both KDE Weather and GNOME Weather installed
   - Verify both appear in the dropdown
   - Verify you can switch between them

### 5. Manual Weather Data Test

Since weather data fetching is not yet implemented, you can test the Bluetooth protocol manually:

1. Create a test in `infinitime/examples/weather.rs`
2. Build and run it with your watch connected
3. Verify the watch displays the weather data

Example test:
```rust
use infinitime::bt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to watch
    let adapter = bluer::Adapter::default().await?;
    let devices = bt::InfiniTime::list_known_devices(&adapter).await?;
    let device = devices.into_iter().next().expect("No InfiniTime device found");
    device.connect().await?;
    let watch = Arc::new(bt::InfiniTime::new(Arc::new(device)).await?);
    
    // Send test weather
    weather::example_send_current_weather(&watch).await?;
    weather::example_send_forecast(&watch).await?;
    
    Ok(())
}
```

## Troubleshooting

### Weather provider not detected

1. Check if the weather app is actually running:
   ```bash
   ps aux | grep -i weather
   ```

2. Check D-Bus services:
   ```bash
   qdbus | grep -i weather  # For KDE
   gdbus call --session --dest org.freedesktop.DBus --object-path /org/freedesktop/DBus --method org.freedesktop.DBus.ListNames | grep -i weather
   ```

3. Check Watchmate logs:
   ```bash
   RUST_LOG=debug cargo run --release 2>&1 | grep -i weather
   ```

### Flatpak service name issues

If using Flatpak, the D-Bus service name might be different. Check:
```bash
flatpak run --command=sh org.kde.kweather
# Inside container:
qdbus | grep kweather
```

Update service names in the code if necessary.

### Watch not receiving weather data

1. Verify your InfiniTime firmware version supports SimpleWeatherService (1.14.0+)
2. Check Bluetooth logs:
   ```bash
   journalctl -f | grep -i bluetooth
   ```

3. Check if the characteristic is available:
   ```bash
   bluetoothctl
   # In bluetoothctl:
   select <adapter>
   devices
   info <watch-address>
   # Should list services including 00050000-78fc-48fe-8e23-433b3a1942d0
   ```

## Next Steps After Testing

Once basic detection and selection is working, the next steps are:

1. **Implement weather data fetching** from the D-Bus interfaces
2. **Add periodic updates** (e.g., every 30 minutes)
3. **Add configuration UI** for update frequency, temperature units, etc.
4. **Handle errors gracefully** when weather data is unavailable
5. **Add user notifications** when weather is updated

## Logging

Enable different log levels for debugging:

```bash
# Info level (default recommended)
RUST_LOG=info cargo run --release

# Debug level (verbose)
RUST_LOG=debug cargo run --release

# Specific module debug
RUST_LOG=infinitime::freedesktop::weather=debug,watchmate=info cargo run --release
```

Look for these log messages:
- "Weather provider started: ..."
- "Weather provider stopped: ..."
- "Weather session started for provider: ..."
- "Restarting provider list update session"

## Contributing Test Results

When testing, please report:
1. Your distribution and version
2. Weather app installation method (native/Flatpak/Snap)
3. Whether detection worked
4. Any error messages
5. D-Bus service names found

This will help improve compatibility across different setups.
