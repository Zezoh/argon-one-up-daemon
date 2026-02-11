# Argon ONE UP UPS Daemon

An efficient Power Management Daemon for the **Argon ONE UP** (Laptop model) on Raspberry Pi, written in Rust.

This daemon replaces the original Python scripts and provides native integration into the system D-Bus, allowing battery status to be displayed directly in desktop environments like **KDE Plasma**, **GNOME**, or **XFCE**.

## Compatibility

* **Raspberry Pi Models**: Pi 3, Pi 4, **Pi 5** (tested)
* **Operating Systems**: Raspberry Pi OS (Bookworm, Trixie), Debian-based distributions
* **Desktop Environments**: XFCE, KDE Plasma, GNOME, or any DE with UPower support

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

### Raspberry Pi 5 Notes

On Raspberry Pi 5, the GPIO controller has changed from BCM2711 to RP1. The daemon automatically detects and uses the correct GPIO chip. If you encounter GPIO-related errors:

1. Verify your user is in the `gpio` group: `sudo usermod -aG gpio $USER`
2. Check that `/dev/gpiochip4` exists: `ls -l /dev/gpiochip*`
3. Ensure I2C is enabled in `raspi-config`

### XFCE Desktop Environment

The daemon integrates with XFCE's power manager through UPower. To see battery status:

1. Add the **Battery Monitor** plugin to your XFCE panel
2. Or use the built-in power manager: `xfce4-power-manager-settings`

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

## License

This project is licensed under the **GPL-3.0 License**. See the `LICENSE` file for the full text.