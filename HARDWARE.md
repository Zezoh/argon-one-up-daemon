# Hardware Configuration Guide

## Overview
This document details all hardware device IDs, GPIO pins, I2C addresses, and register mappings used by the Argon ONE UP daemon.

## Supported Platforms

### Raspberry Pi Models
| Model | Status | GPIO Controller | Notes |
|-------|--------|-----------------|-------|
| **Raspberry Pi 3** | ✅ Supported | BCM2835/BCM2837 | Uses gpiochip0 |
| **Raspberry Pi 4** | ✅ Supported | BCM2711 | Uses gpiochip0 |
| **Raspberry Pi 5** | ✅ Supported | RP1 | Uses gpiochip4 |
| **Compute Module 4** | ✅ Supported | BCM2711 | Same as Pi 4 |
| **Compute Module 5** | ✅ Supported | RP1 | Same as Pi 5, uses gpiochip4 |

### Compute Module 5 (CM5) Compatibility
The CM5 uses the same RP1 GPIO controller as the Raspberry Pi 5:
- **GPIO chip device**: `/dev/gpiochip4`
- **I2C bus**: `/dev/i2c-1` (same as other models)
- **Pin mappings**: Identical to Pi 5
- **rppal library**: Version 0.22.1 supports RP1 controller

## I2C Configuration

### Battery Management IC: CellWise CW2217B
```
Device Address: 0x64 (7-bit address)
I2C Bus: 1 (/dev/i2c-1)
Clock Speed: Standard (100 kHz) or Fast (400 kHz)
```

### CW2217B Register Map
| Register | Address | Function | Size |
|----------|---------|----------|------|
| REG_CONTROL | 0x01 | Control register | 8-bit |
| REG_ICSTATE | 0xA0 | IC state register | 8-bit |
| VCELL_H | 0x02 | Voltage high byte | 8-bit |
| VCELL_L | 0x03 | Voltage low byte | 8-bit |
| SOC_H | 0x04 | State of charge high byte | 8-bit |
| SOC_L | 0x05 | State of charge low byte | 8-bit |
| TEMP_H | 0x06 | Temperature high byte | 8-bit |
| TEMP_L | 0x07 | Temperature low byte | 8-bit |
| CURRENT_H | 0x0E | Current high byte | 8-bit |
| CURRENT_L | 0x0F | Current low byte | 8-bit |

### Sensor Calculations (Per CW2217B Datasheet)

**Voltage (14-bit)**
```rust
const VCELL_LSB_VOLTS: f64 = 305e-6;  // 305 µV per LSB
raw_voltage = (VCELL_H << 8) | VCELL_L
voltage = raw_voltage * 305e-6  // Result in volts
```

**State of Charge (16-bit with fractional)**
```rust
soc = SOC_H + (SOC_L / 256.0)  // Percentage with 1/256% precision
```

**Temperature (16-bit)**
```rust
const TEMP_LSB_SCALE: f64 = 10.0;
const TEMP_OFFSET_CELSIUS: f64 = 40.0;
temp_raw = (TEMP_H << 8) | TEMP_L
temperature = (temp_raw / 10.0) - 40.0  // Result in Celsius
```

**Current (16-bit signed)**
```rust
const CURRENT_SCALE_FACTOR: f64 = 52.4;
const CURRENT_ADC_RESOLUTION: f64 = 32768.0;
const R_SENSE: f64 = 10.0;  // 10 ohm sense resistor
raw_current = (CURRENT_H << 8) | CURRENT_L  // as signed i16
current = (52.4 * raw_current) / (32768.0 * R_SENSE)  // Result in amperes
```

## GPIO Configuration

### Pin Assignments
| Pin Number | Function | Direction | Pull | Notes |
|------------|----------|-----------|------|-------|
| GPIO 4 | Power button | Input | Pull-up | Falling edge trigger |
| GPIO 27 | Lid sensor | Input | Pull-up | Active low |

### GPIO Pin Behavior

**Power Button (GPIO 4)**
- **Active state**: Low (button pressed)
- **Idle state**: High (pull-up)
- **Detection**: Falling edge interrupt
- **Long-press threshold**: 3 seconds (30 × 100ms)
- **Action**: System shutdown via `shutdown -h now`

**Lid Sensor (GPIO 27)**
- **Active state**: Low (lid closed)
- **Idle state**: High (pull-up, lid open)
- **Polling**: Every 2 seconds
- **Action**: Updates D-Bus lid_is_closed property

## Device Access Requirements

### Systemd Service Configuration
The daemon requires access to the following devices:
```ini
DeviceAllow=/dev/i2c-1 rw           # Battery IC communication
DeviceAllow=/dev/gpiochip0 rw       # Pi 3/4
DeviceAllow=/dev/gpiochip1 rw       # Additional GPIO banks
DeviceAllow=/dev/gpiochip2 rw       # Additional GPIO banks
DeviceAllow=/dev/gpiochip3 rw       # Additional GPIO banks
DeviceAllow=/dev/gpiochip4 rw       # Pi 5 / CM5 (RP1 controller)
```

### User Groups
```bash
# Add user to required groups
sudo usermod -aG i2c $USER
sudo usermod -aG gpio $USER
```

## Hardware Verification Commands

### Check I2C Bus and Devices
```bash
# List I2C buses
ls -l /dev/i2c*

# Scan I2C bus 1 for devices (requires i2c-tools)
sudo i2cdetect -y 1
# Should show device at address 0x64
```

### Check GPIO Chips
```bash
# List GPIO chips
ls -l /dev/gpiochip*

# On Pi 5 / CM5, primary chip should be gpiochip4
# On Pi 3/4, primary chip should be gpiochip0

# Check GPIO line information (requires gpiod tools)
sudo gpioinfo
```

### Test GPIO Pins
```bash
# Monitor GPIO 27 (lid sensor)
sudo gpioget gpiochip4 27  # On Pi 5/CM5
sudo gpioget gpiochip0 27  # On Pi 3/4

# Monitor GPIO 4 (power button)
sudo gpioget gpiochip4 4   # On Pi 5/CM5
sudo gpioget gpiochip0 4   # On Pi 3/4
```

## Troubleshooting

### CM5 Specific Issues

**Issue**: GPIO access fails on CM5
**Solution**: 
1. Verify gpiochip4 exists: `ls -l /dev/gpiochip4`
2. Check user is in gpio group: `groups`
3. Ensure systemd service allows gpiochip4 access

**Issue**: I2C communication fails
**Solution**:
1. Enable I2C in raspi-config: `sudo raspi-config` → Interface Options → I2C
2. Verify I2C device exists: `ls -l /dev/i2c-1`
3. Check device is detected: `sudo i2cdetect -y 1`
4. Verify user is in i2c group: `groups`

### Pin Conflicts
GPIO pins 4 and 27 must not be used by other services or device tree overlays. Check for conflicts:
```bash
# Check which GPIO pins are in use
cat /sys/kernel/debug/gpio
```

## References
- **CW2217B Datasheet**: CellWise fuel gauge IC specifications
- **RP1 Peripherals**: Raspberry Pi 5 / CM5 GPIO controller documentation
- **rppal Documentation**: https://docs.rs/rppal/latest/rppal/
- **Argon ONE UP**: Hardware specifications from Argon40
