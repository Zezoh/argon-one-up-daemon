# XFCE4 Battery Display Troubleshooting Guide

**Problem**: Battery not showing on XFCE4 taskbar after following installation instructions.

This guide provides step-by-step troubleshooting to resolve battery display issues in XFCE4.

---

## Quick Diagnostic Checklist

Run through these checks in order. Stop when you find the issue.

### ✅ Step 1: Verify Daemon is Running

Check if the daemon is running:

```bash
sudo systemctl status argon-one-up-daemon
```

**Expected output:**
```
● argon-one-up-daemon.service - Argon ONE UP Laptop Battery and Power Manager
   Loaded: loaded (/etc/systemd/system/argon-one-up-daemon.service; enabled)
   Active: active (running) since ...
```

**If not running**, check for errors:
```bash
sudo journalctl -u argon-one-up-daemon -n 50
```

**Common issues:**
- **"Failed to start"** → Check I2C is enabled: `sudo raspi-config` → Interface Options → I2C
- **"Could not open I2C Bus"** → Check user is in i2c group: `groups | grep i2c`
- **"GPIO Error"** → Check GPIO permissions: `ls -l /dev/gpiochip*`

**Fix: Restart the daemon**
```bash
sudo systemctl restart argon-one-up-daemon
sudo systemctl status argon-one-up-daemon
```

---

### ✅ Step 2: Verify D-Bus Registration

Check if the daemon registered with D-Bus:

```bash
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep UPower
```

**Expected output:**
```
string "org.freedesktop.UPower"
```

**If not present:**
1. Check D-Bus policy is installed:
   ```bash
   ls -l /etc/dbus-1/system.d/org.freedesktop.UPower.BatteryArgon.conf
   ```

2. If missing, install it:
   ```bash
   sudo cp config/org.freedesktop.UPower.BatteryArgon.conf /etc/dbus-1/system.d/
   sudo systemctl reload dbus
   sudo systemctl restart argon-one-up-daemon
   ```

---

### ✅ Step 3: Test Battery Device Presence

Query the battery device directly:

```bash
dbus-send --system --print-reply --dest=org.freedesktop.UPower \
  /org/freedesktop/UPower/devices/battery_argon \
  org.freedesktop.DBus.Properties.GetAll \
  string:"org.freedesktop.UPower.Device"
```

**Expected output:**
Should show battery properties including:
- `Percentage` (0-100)
- `State` (1=Charging, 2=Discharging, 4=Full)
- `Voltage`
- `IsPresent` (should be true)

**If you get an error:**
- "does not exist" → Daemon not registering properly, check logs
- "access denied" → D-Bus policy issue, reinstall policy file

---

### ✅ Step 4: Verify Battery IC Hardware

Check if the CW2217B battery IC is detected:

```bash
sudo i2cdetect -y 1
```

**Expected output:**
You should see `64` in the grid (hex address 0x64).

```
     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
00:          -- -- -- -- -- -- -- -- -- -- -- -- -- 
10: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
20: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
30: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
40: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
50: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
60: -- -- -- -- 64 -- -- -- -- -- -- -- -- -- -- -- 
70: -- -- -- -- -- -- -- --
```

**If `64` is missing:**
- I2C is not enabled → Run `sudo raspi-config`, enable I2C, reboot
- Hardware connection issue → Check Argon ONE UP case connections
- Wrong I2C bus → Try `sudo i2cdetect -y 0` (though bus 1 is standard)

---

### ✅ Step 5: Check for Conflicting UPower

Sometimes a system UPower daemon conflicts with our daemon:

```bash
ps aux | grep -i upower
```

**If you see multiple UPower processes:**

Check which one owns the D-Bus name:
```bash
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.GetNameOwner \
  string:"org.freedesktop.UPower"
```

**To fix conflicts:**
1. Stop system UPower (if installed):
   ```bash
   sudo systemctl stop upower
   sudo systemctl disable upower
   ```

2. Restart our daemon:
   ```bash
   sudo systemctl restart argon-one-up-daemon
   ```

---

### ✅ Step 6: Configure XFCE4 Panel

The daemon might be working, but XFCE4 needs to be configured to display it.

#### Option A: Add Battery Monitor Plugin (Recommended)

1. **Right-click** on the XFCE4 panel
2. Select **Panel** → **Add New Items...**
3. Scroll down and find **"Battery Monitor"** plugin
4. Click **Add**
5. Click **Close**

The battery icon should now appear on your panel.

#### Option B: Configure Existing Plugin

If Battery Monitor is already added but not showing:

1. **Right-click** on the panel
2. Select **Panel** → **Panel Preferences**
3. Go to **Items** tab
4. Find **"Battery Monitor"** in the list
5. Click **Edit** (gear icon)
6. Ensure **"Display percentage"** and **"Display icon"** are checked
7. Click **Close**

---

### ✅ Step 7: Verify XFCE4 Power Manager

Check if XFCE4 power manager is running:

```bash
ps aux | grep xfce4-power-manager
```

**If not running:**
```bash
xfce4-power-manager &
```

**To configure power manager:**
```bash
xfce4-power-manager-settings
```

Ensure the power manager can see the battery:
- Go to **Devices** tab
- Look for "Argon ONE UP" or "battery_argon"

---

### ✅ Step 8: Check System Tray

Sometimes the battery icon appears in the system tray instead of as a panel plugin.

1. Look in your system tray area (notification area)
2. If not visible, add/configure system tray:
   - Right-click panel → **Add New Items**
   - Add **"Notification Area"** or **"Status Notifier Plugin"**

---

## Advanced Troubleshooting

### Check Live Battery Data

Monitor battery updates in real-time:

```bash
sudo journalctl -u argon-one-up-daemon -f
```

You should see periodic updates like:
```
[EVENT] SOC: 85.2% | P: 5.34W | AC: YES | State: Charging
```

### Manual D-Bus Monitor

Monitor D-Bus signals for battery changes:

```bash
dbus-monitor --system "type='signal',interface='org.freedesktop.DBus.Properties',path='/org/freedesktop/UPower/devices/battery_argon'"
```

When battery state changes (plug/unplug AC), you should see signals.

### Restart Desktop Session

Sometimes XFCE needs to be restarted to detect new UPower devices:

1. Save all work
2. Log out
3. Log back in

Or restart XFCE panel:
```bash
xfce4-panel -r
```

### Check Environment Variables

Ensure D-Bus session is properly configured:

```bash
echo $DBUS_SESSION_BUS_ADDRESS
```

Should show something like: `unix:path=/run/user/1000/bus`

---

## Complete Reset Procedure

If nothing else works, try this complete reset:

```bash
# 1. Stop the daemon
sudo systemctl stop argon-one-up-daemon

# 2. Reload D-Bus
sudo systemctl reload dbus

# 3. Restart the daemon
sudo systemctl start argon-one-up-daemon

# 4. Verify registration
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep UPower

# 5. Restart XFCE panel
xfce4-panel -r

# 6. Check Battery Monitor plugin is added to panel
```

---

## Verification Commands Summary

Quick copy-paste verification script:

```bash
echo "=== 1. Daemon Status ==="
sudo systemctl status argon-one-up-daemon

echo -e "\n=== 2. D-Bus Registration ==="
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep UPower

echo -e "\n=== 3. Battery Device ==="
dbus-send --system --print-reply --dest=org.freedesktop.UPower \
  /org/freedesktop/UPower/devices/battery_argon \
  org.freedesktop.DBus.Properties.Get \
  string:"org.freedesktop.UPower.Device" string:"Percentage"

echo -e "\n=== 4. Hardware I2C ==="
sudo i2cdetect -y 1

echo -e "\n=== 5. Recent Logs ==="
sudo journalctl -u argon-one-up-daemon -n 10 --no-pager
```

---

## Common Issues and Solutions

### Issue: "Device at 0x64 not found on I2C bus"

**Solution:**
1. Enable I2C: `sudo raspi-config` → Interface Options → I2C → Enable
2. Reboot: `sudo reboot`
3. Verify: `sudo i2cdetect -y 1`

### Issue: "GPIO Error: Operation not permitted" or "Permission denied accessing GPIO"

This error occurs when the daemon cannot access GPIO devices, typically showing:
```
Error: Failure("GPIO Error: I/O error: Operation not permitted (os error 1)")
```

**Root Causes:**
1. System UPower service is blocking the D-Bus name
2. Insufficient permissions on GPIO/I2C devices
3. Systemd service restrictions preventing device access

**Solution (Step-by-step):**

**Step 1: Stop conflicting UPower service**
```bash
# The system UPower service may be blocking our daemon
sudo systemctl stop upower
sudo systemctl disable upower
```

**Step 2: Ensure device permissions**
```bash
# Check current permissions
ls -l /dev/gpiochip*
ls -l /dev/i2c-1

# On some systems, these may need to be accessible
# Note: The daemon runs as root, so this is usually not needed
# But if DeviceAllow in systemd is blocking access, try:
sudo chmod 666 /dev/gpiochip*
sudo chmod 666 /dev/i2c-1
```

**Step 3: Verify user groups (if running daemon manually)**
```bash
# Add your user to required groups
sudo usermod -aG gpio,i2c $USER
# Log out and back in for group changes to take effect
```

**Step 4: Check systemd service configuration**
```bash
# Verify the service file has correct DeviceAllow entries
cat /etc/systemd/system/argon-one-up-daemon.service | grep DeviceAllow
# Should show:
# DeviceAllow=/dev/i2c-1 rw
# DeviceAllow=/dev/gpiochip0 rw
# DeviceAllow=/dev/gpiochip1 rw
# DeviceAllow=/dev/gpiochip2 rw
# DeviceAllow=/dev/gpiochip3 rw
# DeviceAllow=/dev/gpiochip4 rw
```

**Step 5: Restart the daemon**
```bash
sudo systemctl daemon-reload
sudo systemctl restart argon-one-up-daemon
```

**Step 6: Verify it's working**
```bash
sudo systemctl status argon-one-up-daemon
# Should show: Active: active (running)

# Check logs for any errors
sudo journalctl -u argon-one-up-daemon -n 20
```

**If still failing:**
Check if there's a D-Bus conflict:
```bash
# See what process owns the UPower name
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.GetNameOwner \
  string:"org.freedesktop.UPower"
```

The daemon MUST be the only service claiming `org.freedesktop.UPower` on D-Bus.

### Issue: "D-Bus name already owned by another process"

**Solution:**
```bash
# Check what owns the name
ps aux | grep -i upower

# If system UPower is running, disable it
sudo systemctl stop upower
sudo systemctl disable upower

# Restart our daemon
sudo systemctl restart argon-one-up-daemon
```

### Issue: "Battery shows 0% or wrong values"

**Solution:**
1. Check daemon logs for errors:
   ```bash
   sudo journalctl -u argon-one-up-daemon -n 50
   ```

2. The battery IC might need time to initialize. Wait 30 seconds and check again.

3. Restart the daemon:
   ```bash
   sudo systemctl restart argon-one-up-daemon
   ```

### Issue: "Battery icon shows but no percentage"

**Solution:**
1. Right-click Battery Monitor plugin
2. Select **Properties**
3. Check **"Display percentage"**
4. Check **"Display time remaining"**

---

## Getting Help

If you've tried all these steps and it still doesn't work:

1. **Collect diagnostic information:**
   ```bash
   # Save to a file
   {
     echo "=== System Info ==="
     uname -a
     cat /etc/os-release
     
     echo -e "\n=== Daemon Status ==="
     sudo systemctl status argon-one-up-daemon
     
     echo -e "\n=== Daemon Logs ==="
     sudo journalctl -u argon-one-up-daemon -n 50 --no-pager
     
     echo -e "\n=== D-Bus Registration ==="
     dbus-send --system --print-reply --dest=org.freedesktop.DBus \
       /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep UPower
     
     echo -e "\n=== I2C Detection ==="
     sudo i2cdetect -y 1
     
     echo -e "\n=== D-Bus Policy ==="
     ls -l /etc/dbus-1/system.d/org.freedesktop.UPower.BatteryArgon.conf
     
     echo -e "\n=== Groups ==="
     groups
   } > ~/argon-diagnostic.txt
   
   echo "Diagnostic saved to ~/argon-diagnostic.txt"
   ```

2. **Review the diagnostic file:**
   ```bash
   cat ~/argon-diagnostic.txt
   ```

3. **Share the diagnostic output** when asking for help on forums or GitHub issues.

---

## Success Indicators

You'll know it's working when:

✅ `systemctl status argon-one-up-daemon` shows "active (running)"
✅ D-Bus shows `org.freedesktop.UPower` in the name list
✅ Battery device responds to D-Bus queries with percentage/voltage
✅ `i2cdetect` shows device at address `64`
✅ Battery icon appears in XFCE4 panel
✅ Battery percentage and AC status update correctly
✅ Icon changes when you plug/unplug AC adapter

---

## Quick Reference Card

| Check | Command | Expected Result |
|-------|---------|-----------------|
| Daemon running | `systemctl status argon-one-up-daemon` | Active (running) |
| D-Bus registered | `dbus-send ... ListNames` | org.freedesktop.UPower |
| Battery device | `dbus-send ... GetAll` | Shows properties |
| I2C device | `sudo i2cdetect -y 1` | Shows `64` |
| Panel plugin | Right-click panel → Add Items | Battery Monitor available |
| Live logs | `journalctl -u argon-one-up-daemon -f` | Shows battery events |

---

**Still having issues?** Check the main [README.md](README.md) for installation steps, or review [HARDWARE.md](HARDWARE.md) for hardware specifications.
