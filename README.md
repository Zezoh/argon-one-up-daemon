# Argon ONE UP UPS Daemon

An efficient Power Management Daemon for the **Argon ONE UP** (Laptop model) on Raspberry Pi, written in Rust.

This daemon replaces the original Python scripts and provides native integration into the system D-Bus, allowing battery status to be displayed directly in desktop environments like **KDE Plasma**, **GNOME**, or **XFCE**.

## Compatibility

* **Raspberry Pi Models**: Pi 3, Pi 4, **Pi 5**, Compute Module 4, **Compute Module 5** (CM5)
* **Operating Systems**: Raspberry Pi OS (Bookworm, Trixie), Debian-based distributions
* **Desktop Environments**: XFCE, KDE Plasma, GNOME, or any DE with UPower support
* **GPIO Controllers**: BCM2835, BCM2711, RP1 (auto-detected)

## Features

* **Native D-Bus Integration:** Simulates a UPower-compatible device.
* **Laptop Model Support:** Detects lid status via GPIO 27.
* **Stable AC Detection:** Advanced voltage-based logic to prevent "toggling" artifacts.
* **Power Button Support:** Graceful shutdown on long-press (3 seconds).
* **Optimized Performance:** Uses hardware interrupts for GPIOs and efficient I2C polling.
* **Accurate Sensor Readings:** Full 16-bit precision for voltage, SOC, temperature, and current measurements from CW2217B chip.

## Prerequisites

Before installing, ensure that I2C is enabled on your Raspberry Pi:

1. Run `sudo raspi-config`.
2. Navigate to **Interface Options** -> **I2C** and select **Yes**.
3. Reboot your Pi.

## Installation

### 1. Dependencies

Ensure Rust and Cargo are installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Configure D-Bus Policy

To allow the daemon to reserve the `org.freedesktop.UPower` name on the system bus, copy the policy file to the correct directory:

```bash
sudo cp config/org.freedesktop.UPower.BatteryArgon.conf /etc/dbus-1/system.d/
sudo systemctl reload dbus
```

### 3. Build and Install

Build the optimized release binary and move it to your system path:

```bash
cargo build --release
sudo cp target/release/argon-one-up-daemon /usr/local/bin/
```

### 4. Setup Systemd Service

Install and start the background service:

```bash
sudo cp config/argon-one-up-daemon.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now argon-one-up-daemon
```

## Troubleshooting

### ⚠️ Service Won't Start? (auto-restart loop)

If you see this when checking status:
```bash
sudo systemctl status argon-one-up-daemon
# Shows: Active: activating (auto-restart) (Result: exit-code)
#        Process: ... (code=exited, status=1/FAILURE)
```

**This is caused by the system UPower service blocking the D-Bus name.**

**IMMEDIATE FIX (4 commands):**

```bash
# 1. Stop the conflicting service
sudo systemctl stop upower
sudo systemctl disable upower

# 2. Restart the daemon
sudo systemctl restart argon-one-up-daemon

# 3. Verify it's working
sudo systemctl status argon-one-up-daemon
# Should show: Active: active (running)
```

**For detailed explanation:** See **[GPIO_FIX.md](GPIO_FIX.md)**

---

### 🔋 Battery Not Showing in XFCE4 Taskbar?

**Run the diagnostic script first:**

```bash
bash check-battery.sh
```

This will automatically check:
- Daemon status
- D-Bus registration
- Battery device
- Hardware I2C
- XFCE4 configuration
- Common conflicts

**For detailed troubleshooting:** See **[XFCE4_TROUBLESHOOTING.md](XFCE4_TROUBLESHOOTING.md)**

This comprehensive guide covers:
- Step-by-step diagnostics
- XFCE4 panel configuration
- D-Bus verification
- Common issues and solutions
- Complete diagnostic commands

### Quick Checks

If battery isn't showing, run these commands:

```bash
# 1. Check daemon is running
sudo systemctl status argon-one-up-daemon

# 2. Verify D-Bus registration
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep UPower

# 3. Check battery hardware
sudo i2cdetect -y 1
# Should show device at address 64

# 4. View recent logs
sudo journalctl -u argon-one-up-daemon -n 20
```

### XFCE Desktop Environment

The daemon integrates with XFCE's power manager through UPower. To see battery status:

1. **Right-click** on the XFCE panel
2. Select **Panel** → **Add New Items...**
3. Find and add **"Battery Monitor"** plugin
4. Or use the built-in power manager: `xfce4-power-manager-settings`

### Raspberry Pi 5 and Compute Module 5 (CM5) Notes

On Raspberry Pi 5 and CM5, the GPIO controller has changed from BCM2711 to RP1. The daemon automatically detects and uses the correct GPIO chip. If you encounter GPIO-related errors:

1. Verify your user is in the `gpio` group: `sudo usermod -aG gpio $USER`
2. Check that `/dev/gpiochip4` exists: `ls -l /dev/gpiochip*`
3. Ensure I2C is enabled in `raspi-config`
4. Verify battery IC is detected: `sudo i2cdetect -y 1` (should show device at 0x64)

### Verifying Operation

Check daemon status:
```bash
sudo systemctl status argon-one-up-daemon
```

View logs:
```bash
sudo journalctl -u argon-one-up-daemon -f
```

Test D-Bus integration:
```bash
dbus-send --system --print-reply --dest=org.freedesktop.UPower \
  /org/freedesktop/UPower/devices/battery_argon \
  org.freedesktop.DBus.Properties.GetAll \
  string:"org.freedesktop.UPower.Device"
```

## Documentation

- **[SERVICE_TROUBLESHOOTING.md](SERVICE_TROUBLESHOOTING.md)** - ⚠️ **Service won't start? See this first!**
- **[GPIO_FIX.md](GPIO_FIX.md)** - 🔧 **Quick fix for "GPIO Error: Operation not permitted"**
- **[XFCE4_TROUBLESHOOTING.md](XFCE4_TROUBLESHOOTING.md)** - 🔋 **Battery not showing in XFCE4? Start here!**
- **[QUICK_FIX.md](QUICK_FIX.md)** - ⚡ **Top 5 common issues and fast solutions**
- **[HARDWARE.md](HARDWARE.md)** - Complete hardware configuration guide with all device IDs, GPIO pins, I2C addresses, and register mappings
- **[CODE_EXPLAINED.md](CODE_EXPLAINED.md)** - Detailed code explanation of how the daemon works
- **[INSTALLATION_VERIFICATION.md](INSTALLATION_VERIFICATION.md)** - Installation verification checklist

## License

This project is licensed under the **GPL-3.0 License**. See the `LICENSE` file for the full text.