#!/bin/bash
#
# Argon ONE UP Daemon - XFCE4 Battery Display Diagnostic Script
#
# This script checks all common issues preventing battery display in XFCE4
# Run with: bash check-battery.sh
#

echo "=========================================="
echo "Argon ONE UP Battery Display Diagnostics"
echo "=========================================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check counter
ISSUES=0

# Function to print status
print_ok() {
    echo -e "${GREEN}✓${NC} $1"
}

print_fail() {
    echo -e "${RED}✗${NC} $1"
    ISSUES=$((ISSUES + 1))
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo "  $1"
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Checking Daemon Status..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if systemctl is-active --quiet argon-one-up-daemon; then
    print_ok "Daemon is running"
    
    # Check how long it's been running
    UPTIME=$(systemctl show argon-one-up-daemon --property=ActiveEnterTimestamp --value)
    print_info "Started: $UPTIME"
else
    print_fail "Daemon is NOT running"
    print_info "Try: sudo systemctl start argon-one-up-daemon"
    print_info "Check logs: sudo journalctl -u argon-one-up-daemon -n 20"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. Checking D-Bus Registration..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if dbus-send --system --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames 2>/dev/null | grep -q "org.freedesktop.UPower"; then
    print_ok "D-Bus name 'org.freedesktop.UPower' is registered"
else
    print_fail "D-Bus name 'org.freedesktop.UPower' is NOT registered"
    print_info "Check: ls -l /etc/dbus-1/system.d/org.freedesktop.UPower.BatteryArgon.conf"
    print_info "Install: sudo cp config/org.freedesktop.UPower.BatteryArgon.conf /etc/dbus-1/system.d/"
    print_info "Then: sudo systemctl reload dbus && sudo systemctl restart argon-one-up-daemon"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. Checking Battery Device..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

BATTERY_CHECK=$(dbus-send --system --print-reply --dest=org.freedesktop.UPower \
  /org/freedesktop/UPower/devices/battery_argon \
  org.freedesktop.DBus.Properties.Get \
  string:"org.freedesktop.UPower.Device" string:"Percentage" 2>&1)

if echo "$BATTERY_CHECK" | grep -q "double"; then
    print_ok "Battery device is responding"
    
    # Extract percentage
    PERCENTAGE=$(echo "$BATTERY_CHECK" | grep "double" | awk '{print $3}')
    print_info "Current battery: ${PERCENTAGE}%"
    
    # Check if charging
    STATE_CHECK=$(dbus-send --system --print-reply --dest=org.freedesktop.UPower \
      /org/freedesktop/UPower/devices/battery_argon \
      org.freedesktop.DBus.Properties.Get \
      string:"org.freedesktop.UPower.Device" string:"State" 2>&1)
    
    STATE=$(echo "$STATE_CHECK" | grep "uint32" | awk '{print $3}')
    case $STATE in
        1) print_info "Status: Charging" ;;
        2) print_info "Status: Discharging" ;;
        4) print_info "Status: Full" ;;
        *) print_info "Status: Unknown ($STATE)" ;;
    esac
else
    print_fail "Battery device is NOT responding"
    print_info "Check daemon logs: sudo journalctl -u argon-one-up-daemon -n 30"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4. Checking Hardware (I2C)..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if command -v i2cdetect &> /dev/null; then
    if sudo i2cdetect -y 1 2>/dev/null | grep -q " 64 "; then
        print_ok "Battery IC detected at I2C address 0x64"
    else
        print_fail "Battery IC NOT detected at I2C address 0x64"
        print_info "Enable I2C: sudo raspi-config → Interface Options → I2C"
        print_info "Then reboot and try again"
    fi
else
    print_warn "i2cdetect not found (install i2c-tools to verify hardware)"
    print_info "Install: sudo apt-get install i2c-tools"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5. Checking D-Bus Policy File..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f /etc/dbus-1/system.d/org.freedesktop.UPower.BatteryArgon.conf ]; then
    print_ok "D-Bus policy file is installed"
else
    print_fail "D-Bus policy file is MISSING"
    print_info "Install: sudo cp config/org.freedesktop.UPower.BatteryArgon.conf /etc/dbus-1/system.d/"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6. Checking XFCE4 Panel Configuration..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if pgrep -x "xfce4-panel" > /dev/null; then
    print_ok "XFCE4 panel is running"
    
    # Check if xfce4-power-manager is running
    if pgrep -x "xfce4-power-manager" > /dev/null; then
        print_ok "XFCE4 power manager is running"
    else
        print_warn "XFCE4 power manager is NOT running"
        print_info "Start it: xfce4-power-manager &"
    fi
    
    print_info ""
    print_info "To add Battery Monitor plugin:"
    print_info "  1. Right-click on XFCE panel"
    print_info "  2. Panel → Add New Items..."
    print_info "  3. Find 'Battery Monitor' and click Add"
    
else
    print_warn "XFCE4 panel not detected (might not be running XFCE)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7. Checking for Conflicts..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for system upower
if systemctl is-active --quiet upower 2>/dev/null; then
    print_warn "System UPower service is running (may conflict)"
    print_info "Consider disabling: sudo systemctl stop upower && sudo systemctl disable upower"
elif systemctl list-unit-files | grep -q "^upower.service"; then
    print_ok "System UPower is installed but not running (good)"
else
    print_ok "No conflicting UPower service found"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "8. Recent Daemon Logs..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if systemctl is-active --quiet argon-one-up-daemon; then
    echo "Last 5 log entries:"
    sudo journalctl -u argon-one-up-daemon -n 5 --no-pager | tail -n 5
else
    echo "Daemon not running - showing last error:"
    sudo journalctl -u argon-one-up-daemon -n 10 --no-pager | grep -i "error\|fail" | tail -n 3
fi

echo ""
echo "=========================================="
echo "Diagnostic Summary"
echo "=========================================="

if [ $ISSUES -eq 0 ]; then
    echo -e "${GREEN}✓ No issues detected!${NC}"
    echo ""
    echo "If battery still not showing in XFCE4:"
    echo "  1. Ensure Battery Monitor plugin is added to panel"
    echo "  2. Try restarting XFCE panel: xfce4-panel -r"
    echo "  3. Try logging out and back in"
    echo ""
    echo "See XFCE4_TROUBLESHOOTING.md for detailed guide"
else
    echo -e "${RED}✗ Found $ISSUES issue(s)${NC}"
    echo ""
    echo "Please address the issues above, then run this script again."
    echo ""
    echo "For detailed troubleshooting, see: XFCE4_TROUBLESHOOTING.md"
fi

echo ""
echo "For more help, visit: https://github.com/Zezoh/argon-one-up-daemon"
echo "=========================================="
