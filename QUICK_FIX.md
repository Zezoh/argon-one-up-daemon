# Quick Start: Fix XFCE4 Battery Display Issue

## Problem
Battery not showing on XFCE4 taskbar after installation.

## Solution (3 Steps)

### Step 1: Run Diagnostic Script (30 seconds)

```bash
cd argon-one-up-daemon
bash check-battery.sh
```

This will check:
- ✓ Daemon status
- ✓ D-Bus registration  
- ✓ Battery device
- ✓ Hardware I2C
- ✓ XFCE4 configuration
- ✓ Common conflicts

**The script will tell you exactly what's wrong!**

### Step 2: Follow the Fixes

The script will show specific commands for each issue found.

Common fixes:
- Start daemon: `sudo systemctl start argon-one-up-daemon`
- Enable I2C: `sudo raspi-config` → Interface Options → I2C
- Install D-Bus policy: `sudo cp config/org.freedesktop.UPower.BatteryArgon.conf /etc/dbus-1/system.d/`

### Step 3: Add Battery Monitor to Panel

Even if daemon is working, you must manually add the plugin:

1. **Right-click** on XFCE panel
2. Select **Panel** → **Add New Items...**
3. Find **"Battery Monitor"**
4. Click **Add**

Done! Battery should now appear.

---

## Most Common Issues

### Issue 1: Battery Monitor Plugin Not Added
**This is #1 cause!** The daemon works but XFCE doesn't show it automatically.

**Fix:** Right-click panel → Add New Items → Battery Monitor

### Issue 2: Daemon Not Running
**Check:** `sudo systemctl status argon-one-up-daemon`

**Fix:** `sudo systemctl start argon-one-up-daemon`

### Issue 3: I2C Not Enabled
**Check:** `sudo i2cdetect -y 1` (should show `64`)

**Fix:** 
```bash
sudo raspi-config
# Navigate: Interface Options → I2C → Enable
sudo reboot
```

### Issue 4: D-Bus Policy Missing
**Check:** `ls -l /etc/dbus-1/system.d/org.freedesktop.UPower.BatteryArgon.conf`

**Fix:**
```bash
sudo cp config/org.freedesktop.UPower.BatteryArgon.conf /etc/dbus-1/system.d/
sudo systemctl reload dbus
sudo systemctl restart argon-one-up-daemon
```

---

## Detailed Documentation

For complete troubleshooting, see:
- **XFCE4_TROUBLESHOOTING.md** - 11KB comprehensive guide with all solutions
- **README.md** - Installation and quick checks
- **check-battery.sh** - Automated diagnostic tool

---

## Quick Verification Commands

Copy and paste to check everything:

```bash
# 1. Daemon running?
sudo systemctl status argon-one-up-daemon

# 2. D-Bus registered?
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep UPower

# 3. Battery responding?
dbus-send --system --print-reply --dest=org.freedesktop.UPower \
  /org/freedesktop/UPower/devices/battery_argon \
  org.freedesktop.DBus.Properties.Get \
  string:"org.freedesktop.UPower.Device" string:"Percentage"

# 4. Hardware detected?
sudo i2cdetect -y 1

# 5. Any errors?
sudo journalctl -u argon-one-up-daemon -n 20
```

---

## Success!

You'll know it's working when:
- ✓ Battery icon appears in panel
- ✓ Shows percentage (e.g., "85%")
- ✓ Shows charging/discharging status
- ✓ Updates when AC plugged/unplugged

---

## Still Not Working?

1. Run `bash check-battery.sh` again
2. Read the specific error messages
3. Check **XFCE4_TROUBLESHOOTING.md** for detailed solutions
4. Include the diagnostic output when asking for help

The diagnostic script will tell you exactly what commands to include when reporting issues!
