#![no_std]
#![no_main]

use defmt::error;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_hal::time::{Duration, Instant, Rate};
use esp_println::{self as _, println};

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

const SHT35_ADDRESS: u8 = 0x45;

fn wait(ms: u64) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(ms) {}
}

fn temperature(buffer: [u8; 2]) -> i32 {
    let s_t = ((buffer[0] as u32) << 8 | buffer[1] as u32) as i32;
    -45 + 175 * s_t / (u16::MAX as i32 - 1)
}

fn humidity(buffer: [u8; 2]) -> i32 {
    let s_rh = ((buffer[0] as u32) << 8 | buffer[1] as u32) as i32;
    100 * s_rh / (u16::MAX as i32 - 1)
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));

    let mut i2c_master = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    loop {
        if let Err(e) = i2c_master.write(SHT35_ADDRESS, &[0x24, 0x00]) {
            println!("Error 1: {}", e);
        } else {
            wait(20);

            let mut buffer = [0u8; 6];

            if let Err(e) = i2c_master.read(SHT35_ADDRESS, &mut buffer) {
                println!("Error 2: {}", e);
            } else {
                let t = temperature(buffer[0..2].try_into().unwrap());
                let h = humidity(buffer[2..4].try_into().unwrap());
                println!("Temperature: {}", t);
                println!("Humidity: {}", h);
            }
        }

        wait(500);
    }
}
