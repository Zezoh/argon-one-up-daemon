# GPIO Permission Error - Quick Fix

## Your Error
```
Error: Failure("GPIO Error: I/O error: Operation not permitted (os error 1)")
Feb 11 18:23:49 argon-cm5 systemd[1]: argon-one-up-daemon.service: Failed with result 'exit-code'.
```

## Root Cause
The system UPower service is blocking the D-Bus name `org.freedesktop.UPower` that our daemon needs.

## Solution (4 Commands)

### 1. Stop the conflicting UPower service
```bash
sudo systemctl stop upower
sudo systemctl disable upower
```

### 2. Restart the Argon daemon
```bash
sudo systemctl restart argon-one-up-daemon
```

### 3. Verify it's working
```bash
sudo systemctl status argon-one-up-daemon
```
Expected: `Active: active (running)`

### 4. Check logs for battery updates
```bash
sudo journalctl -u argon-one-up-daemon -n 20
```
Expected: `[EVENT] SOC: 85.2% | P: 5.34W | AC: YES | State: Charging`

## Why This Works

The daemon needs to claim the D-Bus name `org.freedesktop.UPower` to provide battery information to your desktop. However, Raspberry Pi OS comes with a system UPower service that claims this name first.

When two services try to claim the same D-Bus name:
1. System UPower starts and claims the name
2. Our daemon tries to start
3. Our daemon can't claim the name (already taken)
4. D-Bus registration fails
5. GPIO initialization fails as a side effect
6. You see "GPIO Error: Operation not permitted"

By stopping the system UPower service, our daemon can successfully claim the D-Bus name and start properly.

## Verification

After running the commands above, verify everything works:

```bash
# Check daemon is running
sudo systemctl status argon-one-up-daemon

# Check D-Bus registration
dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep UPower

# Test battery device
dbus-send --system --print-reply --dest=org.freedesktop.UPower \
  /org/freedesktop/UPower/devices/battery_argon \
  org.freedesktop.DBus.Properties.Get \
  string:"org.freedesktop.UPower.Device" string:"Percentage"
```

All three should succeed.

## Next Steps

1. **Add Battery Monitor to XFCE Panel:**
   - Right-click on panel
   - Panel → Add New Items...
   - Find "Battery Monitor"
   - Click Add

2. **Run diagnostic to verify:**
   ```bash
   bash check-battery.sh
   ```
   Should show 0 issues!

## Automatic Detection

The updated `check-battery.sh` script now automatically detects this GPIO error and shows the fix commands. Next time you encounter this issue, the diagnostic script will guide you through the solution.

## More Help

- See **XFCE4_TROUBLESHOOTING.md** for complete troubleshooting guide
- See **QUICK_FIX.md** for common issues and fixes
- Run `bash check-battery.sh` for automated diagnostics
