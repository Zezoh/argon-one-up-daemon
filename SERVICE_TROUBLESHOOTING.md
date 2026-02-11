# Service Startup Troubleshooting

## Symptom: Daemon Won't Start

When you run:
```bash
sudo systemctl status argon-one-up-daemon
```

You see one of these outputs:

### ❌ Symptom 1: Auto-Restart Loop

```
Active: activating (auto-restart) (Result: exit-code)
Process: 12531 ExecStart=/usr/local/bin/argon-one-up-daemon (code=exited, status=1/FAILURE)
Main PID: 12531 (code=exited, status=1/FAILURE)
```

**Problem:** Service crashes immediately after starting and systemd keeps retrying.

**Fix:** This is the UPower D-Bus conflict. Apply the fix:

```bash
sudo systemctl stop upower
sudo systemctl disable upower
sudo systemctl restart argon-one-up-daemon
sudo systemctl status argon-one-up-daemon
```

Expected result: `Active: active (running)`

**Why:** System UPower owns the D-Bus name our daemon needs. See [GPIO_FIX.md](GPIO_FIX.md) for details.

---

### ❌ Symptom 2: Service is Stopped

```
Active: inactive (dead)
```

**Problem:** Service is not enabled or has been stopped.

**Fix:**

```bash
sudo systemctl enable argon-one-up-daemon
sudo systemctl start argon-one-up-daemon
sudo systemctl status argon-one-up-daemon
```

Expected result: `Active: active (running)`

---

### ❌ Symptom 3: Service is Running but Battery Not Showing

```
Active: active (running)
```

But battery icon doesn't appear in taskbar.

**Problem:** XFCE panel plugin not added, or D-Bus not working.

**Fix:**

1. **Run diagnostic:**
   ```bash
   bash check-battery.sh
   ```

2. **Add Battery Monitor plugin to XFCE panel:**
   - Right-click panel → Add New Items → Battery Monitor

See [XFCE4_TROUBLESHOOTING.md](XFCE4_TROUBLESHOOTING.md) for detailed steps.

---

## Quick Diagnostic Decision Tree

```
systemctl status argon-one-up-daemon shows...

┌─ "activating (auto-restart)" + "exit-code"
│  └─> UPower conflict
│     └─> Solution: GPIO_FIX.md (stop system upower)
│
┌─ "inactive (dead)"
│  └─> Service not started
│     └─> Solution: systemctl enable/start
│
┌─ "active (running)"
│  └─> Daemon working, but battery not showing
│     └─> Solution: Run check-battery.sh
│        └─> Add XFCE panel plugin
│
└─ Other error messages
   └─> Check logs: sudo journalctl -u argon-one-up-daemon -n 50
      └─> See XFCE4_TROUBLESHOOTING.md
```

---

## Common Error Messages and Solutions

### Error: "GPIO Error: Operation not permitted"

**Full error in logs:**
```
Error: Failure("GPIO Error: I/O error: Operation not permitted (os error 1)")
```

**Cause:** System UPower is blocking the D-Bus name.

**Solution:** See [GPIO_FIX.md](GPIO_FIX.md)

```bash
sudo systemctl stop upower && sudo systemctl disable upower
sudo systemctl restart argon-one-up-daemon
```

---

### Error: "Could not open I2C Bus"

**Cause:** I2C is not enabled.

**Solution:**
```bash
sudo raspi-config
# Navigate: Interface Options → I2C → Enable
sudo reboot
```

---

### Error: "Failed to register D-Bus service"

**Cause:** D-Bus policy file not installed or system UPower conflict.

**Solution:**
```bash
# Install policy file
sudo cp config/org.freedesktop.UPower.BatteryArgon.conf /etc/dbus-1/system.d/
sudo systemctl reload dbus

# Stop conflicting service
sudo systemctl stop upower
sudo systemctl disable upower

# Restart daemon
sudo systemctl restart argon-one-up-daemon
```

---

## Verification Steps

After applying any fix, verify everything works:

### 1. Check Service Status
```bash
sudo systemctl status argon-one-up-daemon
```
Expected: `Active: active (running)` (green)

### 2. Check Logs
```bash
sudo journalctl -u argon-one-up-daemon -n 10
```
Expected: Battery events like `[EVENT] SOC: 85.2% | P: 5.34W | AC: YES`

### 3. Run Full Diagnostic
```bash
bash check-battery.sh
```
Expected: `✓ No issues detected!`

### 4. Verify D-Bus
```bash
dbus-send --system --print-reply --dest=org.freedesktop.UPower \
  /org/freedesktop/UPower/devices/battery_argon \
  org.freedesktop.DBus.Properties.Get \
  string:"org.freedesktop.UPower.Device" string:"Percentage"
```
Expected: Shows battery percentage value

### 5. Check Battery Icon
- Should appear in XFCE panel (if Battery Monitor plugin added)
- Should show percentage and charging status

---

## Still Having Issues?

1. **Run the automated diagnostic:**
   ```bash
   bash check-battery.sh
   ```
   This will identify the specific problem and provide fix commands.

2. **Check comprehensive guides:**
   - [GPIO_FIX.md](GPIO_FIX.md) - UPower conflict (most common)
   - [QUICK_FIX.md](QUICK_FIX.md) - Top 5 common issues
   - [XFCE4_TROUBLESHOOTING.md](XFCE4_TROUBLESHOOTING.md) - Complete guide

3. **Collect diagnostic info:**
   ```bash
   {
     echo "=== System Info ==="
     uname -a
     echo ""
     echo "=== Service Status ==="
     sudo systemctl status argon-one-up-daemon
     echo ""
     echo "=== Recent Logs ==="
     sudo journalctl -u argon-one-up-daemon -n 30 --no-pager
     echo ""
     echo "=== I2C Detection ==="
     sudo i2cdetect -y 1
   } > ~/argon-diagnostic.txt
   
   cat ~/argon-diagnostic.txt
   ```

4. **Report the issue** with the diagnostic output.

---

## Quick Command Reference

| Issue | Command | Expected Result |
|-------|---------|-----------------|
| Check status | `sudo systemctl status argon-one-up-daemon` | `active (running)` |
| View logs | `sudo journalctl -u argon-one-up-daemon -n 20` | Battery events |
| Stop UPower | `sudo systemctl stop upower && sudo systemctl disable upower` | Service stopped |
| Restart daemon | `sudo systemctl restart argon-one-up-daemon` | Service restarts |
| Run diagnostic | `bash check-battery.sh` | `0 issues found` |
| Check I2C | `sudo i2cdetect -y 1` | Shows `64` |
| Test D-Bus | `dbus-send ... ListNames \| grep UPower` | Shows name |
