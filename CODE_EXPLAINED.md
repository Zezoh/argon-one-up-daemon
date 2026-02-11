# Code Explanation - Argon ONE UP Daemon

## What Does This Code Do?

This Rust daemon provides battery management and power control for the Argon ONE UP laptop case. It continuously monitors battery status from the CW2217B fuel gauge IC and exposes this information to the desktop environment via the standard UPower D-Bus interface, allowing battery indicators to work just like on a real laptop.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop Environment                       │
│              (XFCE, KDE Plasma, GNOME, etc.)                │
└────────────────────────┬────────────────────────────────────┘
                         │ Reads battery info via UPower API
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   D-Bus System Bus                           │
│            (org.freedesktop.UPower interface)               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│               Argon ONE UP Daemon (This Code)               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Main Loop (every 2 seconds):                        │   │
│  │ 1. Read battery metrics from CW2217B via I2C        │   │
│  │ 2. Read GPIO states (lid, power button)             │   │
│  │ 3. Update D-Bus properties                          │   │
│  │ 4. Send D-Bus change signals to desktop             │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Power Button Thread:                                │   │
│  │ - Monitor GPIO 4 for button press                   │   │
│  │ - Execute shutdown on 3-second hold                 │   │
│  └─────────────────────────────────────────────────────┘   │
└──────────────┬─────────────────────────┬────────────────────┘
               │                         │
               ▼                         ▼
┌──────────────────────┐    ┌───────────────────────┐
│   I2C Bus 1          │    │  GPIO Pins            │
│   (CW2217B @ 0x64)   │    │  - Pin 4: Button      │
│                      │    │  - Pin 27: Lid        │
└──────────────────────┘    └───────────────────────┘
```

## Main Components

### 1. Hardware Manager (`HardwareManager` struct)

**Purpose**: Communicates with the CW2217B battery fuel gauge chip via I2C.

**Key Methods**:

- `new()` - Initializes I2C bus connection to address 0x64
- `init()` - Activates the CW2217B chip (required after power-up or reset)
- `read_byte(reg)` - Reads a single byte from a CW2217B register
- `update_status()` - Reads all battery metrics and calculates derived values

**What it reads**:
1. **Voltage** (VCELL_H/L registers): Battery voltage in volts
2. **State of Charge** (SOC_H/L registers): Battery percentage
3. **Temperature** (TEMP_H/L registers): Battery temperature in Celsius
4. **Current** (CURRENT_H/L registers): Charge/discharge current in amperes

**How it calculates**:
- Voltage: Combines 14-bit value from two registers, multiplies by 305 µV
- SOC: Integer percentage + fractional part (1/256% resolution)
- Temperature: 16-bit raw value / 10 - 40°C offset
- Current: Signed 16-bit value scaled by datasheet formula
- Power: voltage × current
- AC status: Detected by positive current (charging)

### 2. D-Bus Interfaces

#### UPowerManager Interface (`/org/freedesktop/UPower`)

**Purpose**: Simulates the main UPower daemon that desktop environments query.

**Provides**:
- `enumerate_devices()` - Lists available battery devices
- `get_display_device()` - Returns primary battery device path
- `on_battery` property - Whether system is running on battery (AC unplugged)
- `lid_is_closed` property - Whether laptop lid is closed
- `lid_is_present` property - Always true (laptop has a lid)

#### ArgonBattery Interface (`/org/freedesktop/UPower/devices/battery_argon`)

**Purpose**: Represents the battery itself with all its properties.

**Provides**:
- `percentage` - Battery charge level (0-100%)
- `voltage` - Current voltage in volts
- `state` - Charging (1), Discharging (2), or Full (4)
- `energy` - Current energy in Wh
- `energy_full` - Full capacity (55.21 Wh)
- `energy_rate` - Power consumption/charging rate in watts
- `temperature` - Battery temperature
- `model` - "Argon ONE UP"
- `vendor` - "Argon40"
- `technology` - Li-ion (1)

### 3. Main Event Loop

**Runs every 2 seconds**:

1. **Read hardware state**:
   ```rust
   hw.update_status(lid_pin.is_low())
   ```
   - Reads all CW2217B registers via I2C
   - Checks GPIO 27 for lid status
   - Calculates voltage, SOC, temperature, current, power
   - Detects AC presence (charging current > 0)

2. **Check for changes**:
   - Compares new state with last broadcast state
   - Considers change significant if:
     - Charging state changed (charging ↔ discharging ↔ full)
     - AC status changed (plugged ↔ unplugged)
     - Lid status changed (open ↔ closed)
     - SOC changed by > 0.5%
     - Power changed by > 0.05W

3. **Broadcast updates**:
   ```rust
   battery_iface.percentage_changed()
   battery_iface.state_changed()
   manager_iface.on_battery_changed()
   manager_iface.lid_is_closed_changed()
   ```
   - Sends D-Bus property change signals
   - Desktop environment receives notifications
   - Battery icon updates automatically

4. **Log events**:
   ```
   [EVENT] SOC: 85.2% | P: 5.34W | AC: YES | State: Charging
   ```

### 4. Power Button Monitor Thread

**Runs independently in background**:

```rust
thread::spawn(move || {
    // Monitor GPIO 4 for falling edge (button press)
    if shutdown_pin.is_low() {
        // Count how long button is held
        while shutdown_pin.is_low() && dur < 50 {
            dur += 1;  // Increment every 100ms
            thread::sleep(Duration::from_millis(100));
        }
        // If held for 3 seconds (30 × 100ms), shut down
        if dur >= 30 {
            Command::new("shutdown").args(["-h", "now"]).spawn();
        }
    }
})
```

**Purpose**: Provides graceful shutdown when power button is held for 3+ seconds.

## Data Flow Example

### When AC Adapter is Plugged In

1. **Hardware**: Current flows into battery, CURRENT_H/L registers show positive value
2. **Daemon reads**: `current = +1.2A` (positive = charging)
3. **AC detection**: `ac_present = true` (debounced over 3 samples)
4. **State determination**: 
   - If SOC < 98%: `state = 1` (Charging)
   - If SOC >= 98%: `state = 4` (Full)
5. **D-Bus signals sent**:
   - `on_battery_changed` → `false`
   - `state_changed` → `Charging`
6. **Desktop environment**: 
   - Battery icon shows AC plugged in
   - Shows "Charging" status
   - May show time to full

### When Battery is Discharging

1. **Hardware**: Current flows out of battery, CURRENT_H/L registers show negative value
2. **Daemon reads**: `current = -0.8A` (negative = discharging)
3. **AC detection**: `ac_present = false` (no charging current)
4. **State determination**: `state = 2` (Discharging)
5. **Power calculation**: `power = voltage × |current| = 11.1V × 0.8A = 8.88W`
6. **D-Bus signals sent**:
   - `on_battery_changed` → `true`
   - `state_changed` → `Discharging`
   - `energy_rate_changed` → `8.88W`
7. **Desktop environment**:
   - Battery icon shows discharging
   - Shows remaining time estimate
   - May show low battery warnings if SOC < 10%

### When Lid is Closed

1. **Hardware**: GPIO 27 pulled low by lid sensor switch
2. **Daemon reads**: `lid_pin.is_low() = true`
3. **State update**: `lid_closed = true`
4. **D-Bus signal sent**: `lid_is_closed_changed` → `true`
5. **Desktop environment**:
   - Triggers suspend/sleep (if configured)
   - Or displays "lid closed" notification

## Error Handling

The daemon uses graceful error handling to prevent crashes:

### I2C Communication Errors
```rust
fn read_byte(&mut self, reg: u8) -> u8 {
    if self.i2c.write(&[reg]).is_err() { return 255; }  // Invalid value
    if self.i2c.read(&mut res).is_err() { return 255; }
    res[0]
}
```
If I2C fails, returns 0xFF which is detected and handled.

### GPIO Access Errors
```rust
let gpio = Gpio::new().map_err(|e| {
    error!("GPIO initialization error: {}", e);
    zbus::Error::Failure(format!("GPIO Error: {}", e))
})?;
```
Logs error and returns gracefully instead of panicking.

### RwLock Poisoning
```rust
self.state.read().map(|s| s.soc).unwrap_or(0.0)
```
If lock is poisoned, returns safe default value.

## Logging

Uses structured logging via `log` and `env_logger`:

- `debug!()` - Detailed sensor readings (only with --debug flag)
- `info!()` - Important events (startup, sync, state changes)
- `warn!()` - Non-critical issues (hardware timeouts, shutdown signal)
- `error!()` - Critical failures (GPIO/I2C initialization)

View logs: `journalctl -u argon-one-up-daemon -f`

## Performance

- **CPU usage**: < 0.5% (mostly idle, wakes every 2 seconds)
- **Memory**: ~5 MB resident
- **I2C reads**: 7 bytes every 2 seconds
- **D-Bus signals**: Only sent when values change
- **GPIO interrupts**: Hardware-triggered (no polling for button)

## Security

- **Runs as root**: Required for GPIO, I2C, and D-Bus system bus access
- **Systemd restrictions**: 
  - `ProtectSystem=strict` - Cannot write to system directories
  - `ProtectHome=true` - Cannot access user files
  - `PrivateTmp=true` - Isolated /tmp
  - `DeviceAllow` - Only access specific I2C and GPIO devices
- **No network access**: Does not open any network connections
- **Shutdown capability**: By design - power button should shut down system

## Summary

This daemon creates a seamless laptop experience by:
1. **Reading** battery data from hardware via I2C
2. **Monitoring** GPIO pins for lid and button events
3. **Translating** to standard UPower D-Bus interface
4. **Enabling** any Linux desktop environment to display battery status
5. **Providing** safe power button shutdown functionality

The code is designed to be robust, efficient, and compatible across multiple Raspberry Pi models including the latest CM5.
