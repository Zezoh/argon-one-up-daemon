// Argon ONE UP Laptop Daemon (Integrated Version)
// License: GPL-3.0

const VERSION: &str = "V6.5";

use rppal::i2c::I2c;
use rppal::gpio::{Gpio, Trigger};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use std::env;
use std::collections::VecDeque;
use std::process::Command;
use zbus::{interface, connection::Builder};
use log::{info, warn, error, debug};
use env_logger;

// Hardware constants for the Argon ONE UP (Laptop Model)
const R_SENSE: f64 = 10.0;
const PIN_LID: u8 = 27;

// CW2217B Datasheet Constants
const VCELL_LSB_VOLTS: f64 = 305e-6;  // 305 µV per LSB
const TEMP_LSB_SCALE: f64 = 10.0;     // Temperature resolution: 0.1°C per LSB
const TEMP_OFFSET_CELSIUS: f64 = 40.0; // Temperature offset per datasheet
const CURRENT_SCALE_FACTOR: f64 = 52.4; // Current calculation scaling factor from datasheet
const CURRENT_ADC_RESOLUTION: f64 = 32768.0; // 15-bit signed ADC resolution

// IC: CellWise CW2217B (CW2217BAAD)
const ADDR_BATTERY: u8 = 0x64;
const REG_CONTROL: u8 = 0x01;
const REG_ICSTATE: u8 = 0xA0;
const PIN_SHUTDOWN: u8 = 4;
const VCELL_H: u8 = 0x02;
const VCELL_L: u8 = 0x03;
const SOC_H: u8 = 0x04;
const SOC_L: u8 = 0x05;
const TEMP_H: u8 = 0x06;
const TEMP_L: u8 = 0x07;
const CURRENT_H: u8 = 0x0E;
const CURRENT_L: u8 = 0x0F;


#[derive(Clone, Copy, Debug, PartialEq)]
struct BatteryState {
    soc: f64,
    voltage: f64,
    current: f64,
    temperature: f64,
    power: f64,
    state: u32,     // 1=Charging, 2=Discharging, 4=Full
    ac_present: bool,
    lid_closed: bool,
}

// --- UPower Manager Interface ---
struct UPowerManager {
    state: Arc<RwLock<BatteryState>>,
}

#[interface(name = "org.freedesktop.UPower")]
impl UPowerManager {
    fn enumerate_devices(&self) -> Vec<zbus::zvariant::OwnedObjectPath> {
        const BATTERY_PATH: &str = "/org/freedesktop/UPower/devices/battery_argon";
        vec![
            // This static string is known to be valid, so expect() is safe here
            zbus::zvariant::ObjectPath::from_static_str(BATTERY_PATH)
                .expect("Static object path is valid")
                .into()
        ]
    }

    fn get_display_device(&self) -> zbus::zvariant::OwnedObjectPath {
        const BATTERY_PATH: &str = "/org/freedesktop/UPower/devices/battery_argon";
        // This static string is known to be valid, so expect() is safe here
        zbus::zvariant::ObjectPath::from_static_str(BATTERY_PATH)
            .expect("Static object path is valid")
            .into()
    }

    #[zbus(property)]
    fn daemon_version(&self) -> String {
        "0.99.11".to_string()
    }

    #[zbus(property)]
    fn on_battery(&self) -> bool {
        !self.state.read().map(|s| s.ac_present).unwrap_or(false)
    }

    #[zbus(property)]
    fn lid_is_closed(&self) -> bool {
        self.state.read().map(|s| s.lid_closed).unwrap_or(false)
    }

    #[zbus(property)]
    fn lid_is_present(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn critical_action(&self) -> String {
        "PowerOff".to_string()
    }
}

// --- UPower Device Interface ---
struct ArgonBattery {
    state: Arc<RwLock<BatteryState>>,
}

#[interface(name = "org.freedesktop.UPower.Device")]
impl ArgonBattery {
    #[zbus(property)]
    fn percentage(&self) -> f64 {
        self.state.read().map(|s| s.soc).unwrap_or(0.0)
    }

    #[zbus(property)]
    fn voltage(&self) -> f64 {
        self.state.read().map(|s| s.voltage).unwrap_or(0.0)
    }

    #[zbus(property)]
    fn energy(&self) -> f64 {
        self.state.read().map(|s| s.soc * 0.5521).unwrap_or(0.0)
    }

    #[zbus(property)]
    fn energy_full(&self) -> f64 {
        55.21
    }

    #[zbus(property)]
    fn energy_full_design(&self) -> f64 {
        55.21
    }

    #[zbus(property)]
    fn energy_rate(&self) -> f64 {
        self.state.read().map(|s| s.power).unwrap_or(0.0)
    }

    #[zbus(property)]
    fn state(&self) -> u32 {
        self.state.read().map(|s| s.state).unwrap_or(0)
    }

    #[zbus(property)]
    fn is_present(&self) -> bool { true }

    #[zbus(property)]
    fn is_rechargeable(&self) -> bool { true }

    #[zbus(property)]
    fn power_supply(&self) -> bool { true }

    #[zbus(property)]
    fn technology(&self) -> u32 { 1 } // 1 = Li-ion

    #[zbus(property)]
    fn model(&self) -> String { "Argon ONE UP".to_string() }

    #[zbus(property)]
    fn vendor(&self) -> String { "Argon40".to_string() }

    #[zbus(property)]
    fn type_(&self) -> u32 { 2 } // 2 = Battery
}

struct HardwareManager {
    i2c: I2c,
    ac_history: VecDeque<bool>,
}

impl HardwareManager {
    fn new(_debug: bool) -> Result<Self, String> {
        let mut i2c = I2c::with_bus(1).map_err(|e| format!("Could not open I2C Bus 1: {}", e))?;
        i2c.set_slave_address(ADDR_BATTERY as u16).map_err(|e| format!("Could not set I2C slave address: {}", e))?;
        Ok(HardwareManager { i2c, ac_history: VecDeque::new() })
    }

    fn init(&mut self) -> bool {
        debug!("Initiating CW2217B controller activation...");
        let mut retries = 3;

        while retries > 0 {
            retries -= 1;
            let _ = self.i2c.write(&[REG_CONTROL, 0x30]);
            thread::sleep(Duration::from_millis(500));
            let _ = self.i2c.write(&[REG_CONTROL, 0x00]);
            thread::sleep(Duration::from_millis(500));

            let mut wait_secs = 5;
            while wait_secs > 0 {
                let status = self.read_byte(REG_ICSTATE);
                if status != 255 && status != 0 && (status & 0x0C) != 0 {
                    debug!("CW2217B Active. State: 0x{:02X}", status);
                    return true;
                }
                thread::sleep(Duration::from_secs(1));
                wait_secs -= 1;
            }
        }
        false
    }

    fn read_byte(&mut self, reg: u8) -> u8 {
        let mut res = [0u8; 1];
        if self.i2c.write(&[reg]).is_err() { return 255; }
        thread::sleep(Duration::from_millis(20));
        if self.i2c.read(&mut res).is_err() { return 255; }
        res[0]
    }

    fn update_status(&mut self, lid_is_low: bool) -> Option<BatteryState> {


        let v_raw_h = self.read_byte(VCELL_H);
        let v_raw_l = self.read_byte(VCELL_L);
        let soc_raw_high = self.read_byte(SOC_H);
        let soc_raw_low = self.read_byte(SOC_L);
        let temp_h = self.read_byte(TEMP_H);
        let temp_l = self.read_byte(TEMP_L);
        let current_raw_high = self.read_byte(CURRENT_H);
        let current_raw_low = self.read_byte(CURRENT_L);

        if (v_raw_h == 255 || v_raw_h == 0) && (soc_raw_high == 255 || soc_raw_high == 0) {
            return None;
        }

        // Fix voltage calculation - use both high and low bytes (14-bit value)
        let raw_voltage = ((v_raw_h as u16) << 8) | (v_raw_l as u16);
        let voltage = raw_voltage as f64 * VCELL_LSB_VOLTS;

        // Fix SOC calculation - use both high and low bytes
        let soc = (soc_raw_high as f64) + (soc_raw_low as f64 / 256.0);
        let soc = soc.clamp(0.0, 100.0);

        // Process Current (Signed 16-bit integer)
        let raw_current = (((current_raw_high as u16) << 8) | (current_raw_low as u16)) as i16;

        // Fix temperature calculation - use both high and low bytes (16-bit value)
        let temp_raw = ((temp_h as u16) << 8) | (temp_l as u16);
        let temperature = temp_raw as f64 / TEMP_LSB_SCALE - TEMP_OFFSET_CELSIUS;

        // CW2217B Current Calculation per datasheet
        let current = (CURRENT_SCALE_FACTOR * raw_current as f64) / (CURRENT_ADC_RESOLUTION * R_SENSE);
        let power = (voltage * current).abs();

        // AC DETECTION LOGIC:
        let raw_ac = current > 0.0;

        // Debounce AC detection
        self.ac_history.push_back(raw_ac);
        if self.ac_history.len() > 3 { self.ac_history.pop_front(); }
        let ac_present = self.ac_history.iter().filter(|&&x| x).count() >= 2;

        let state = if !ac_present {
            2 // Discharging
        } else if current > 0.05 {
            1 // Charging
        } else if soc >= 98.0 {
            4 // Full
        } else {
            1 // Charging Fallback
        };

        debug!("CW2217B -> V: {:.2}V | I: {:.3}A | P: {:.2}W | SOC: {:.1}% | Temperature: {:.1}°C | AC: {} | Lid: {}",
                     voltage, current, power, soc, temperature,
                     if ac_present { "YES" } else { "NO" },
                     if lid_is_low { "CLOSED" } else { "OPEN" });

        Some(BatteryState {
            soc,
            voltage,
            current,
            temperature,
            power,
            state,
            ac_present,
            lid_closed: lid_is_low,
        })
    }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    let debug_mode = args.iter().any(|arg| arg == "--debug" || arg == "-d");

    info!("--- Argon ONE UP Rust Manager ({}) ---", VERSION);

    let gpio = Gpio::new().map_err(|e| {
        error!("GPIO initialization error: {}", e);
        zbus::Error::Failure(format!("GPIO Error: {}", e))
    })?;
    let mut hw = HardwareManager::new(debug_mode).map_err(|e| {
        error!("Hardware initialization error: {}", e);
        zbus::Error::Failure(e)
    })?;

    thread::sleep(Duration::from_millis(500));
    if !hw.init() {
        warn!("Hardware activation sequence timed out.");
    }

    let lid_pin = gpio.get(PIN_LID).map_err(|e| {
        error!("Failed to get lid pin: {}", e);
        zbus::Error::Failure(format!("Failed to get lid pin: {}", e))
    })?.into_input_pullup();

    let initial_state = loop {
        if let Some(state) = hw.update_status(lid_pin.is_low()) {
            info!("Battery controller synchronized.");
            break state;
        }
        hw.init();
        thread::sleep(Duration::from_secs(2));
    };

    let shared_state = Arc::new(RwLock::new(initial_state));
    let last_broadcast_state = Arc::new(RwLock::new(initial_state));

    let battery_obj = ArgonBattery { state: Arc::clone(&shared_state) };
    let manager_obj = UPowerManager { state: Arc::clone(&shared_state) };

    let conn = Builder::system()?
        .name("org.freedesktop.UPower")?
        .serve_at("/org/freedesktop/UPower", manager_obj)?
        .serve_at("/org/freedesktop/UPower/devices/battery_argon", battery_obj)?
        .build()
        .await?;

    let object_server = conn.object_server();
    let battery_iface = object_server
        .interface::<_, ArgonBattery>("/org/freedesktop/UPower/devices/battery_argon")
        .await?;
    let manager_iface = object_server
        .interface::<_, UPowerManager>("/org/freedesktop/UPower")
        .await?;

    thread::spawn(move || {
        let gpio_btn = match Gpio::new() {
            Ok(g) => g,
            Err(e) => {
                error!("Failed to initialize GPIO for shutdown button: {}", e);
                return;
            }
        };
        let mut shutdown_pin = match gpio_btn.get(PIN_SHUTDOWN) {
            Ok(pin) => pin.into_input_pullup(),
            Err(e) => {
                error!("Failed to get shutdown pin: {}", e);
                return;
            }
        };
        let _ = shutdown_pin.set_interrupt(Trigger::FallingEdge, Some(Duration::from_millis(10)));
        loop {
            if let Ok(Some(_)) = shutdown_pin.poll_interrupt(true, Some(Duration::from_millis(500))) {
                if shutdown_pin.is_low() {
                    let mut dur = 0;
                    while shutdown_pin.is_low() && dur < 50 {
                        dur += 1;
                        thread::sleep(Duration::from_millis(100));
                    }
                    if dur >= 30 {
                        warn!("Soft-shutdown signal detected!");
                        let _ = Command::new("shutdown").args(["-h", "now"]).spawn();
                    }
                }
            }
        }
    });

    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;

        if let Some(new_state) = hw.update_status(lid_pin.is_low()) {
            let mut changed = false;
            {
                if let (Ok(mut w), Ok(last)) = (shared_state.write(), last_broadcast_state.read()) {
                    if new_state.state != last.state ||
                       new_state.ac_present != last.ac_present ||
                       new_state.lid_closed != last.lid_closed ||
                       (new_state.soc - last.soc).abs() > 0.5 ||
                       (new_state.power - last.power).abs() > 0.05
                    {
                        changed = true;
                    }
                    *w = new_state;
                } else {
                    error!("Failed to acquire lock on shared state");
                    continue;
                }
            }

            if changed {
                if let Ok(mut last_w) = last_broadcast_state.write() {
                    *last_w = new_state;
                } else {
                    error!("Failed to acquire lock on last_broadcast_state");
                    continue;
                }

                let bat_ctx = battery_iface.signal_context();
                let bat_inst = battery_iface.get().await;
                let _ = bat_inst.percentage_changed(bat_ctx).await;
                let _ = bat_inst.state_changed(bat_ctx).await;
                let _ = bat_inst.voltage_changed(bat_ctx).await;
                let _ = bat_inst.energy_changed(bat_ctx).await;
                let _ = bat_inst.energy_rate_changed(bat_ctx).await;

                let mgr_ctx = manager_iface.signal_context();
                let mgr_inst = manager_iface.get().await;
                let _ = mgr_inst.on_battery_changed(mgr_ctx).await;
                let _ = mgr_inst.lid_is_closed_changed(mgr_ctx).await;

                info!("[EVENT] SOC: {:.1}% | P: {:.2}W | AC: {} | State: {}",
                    new_state.soc,
                    new_state.power,
                    if new_state.ac_present { "YES" } else { "NO" },
                    match new_state.state { 1 => "Charging", 2 => "Discharging", 4 => "Full", _ => "Unknown" }
                );
            }
        } else {
            hw.init();
        }
    }
}