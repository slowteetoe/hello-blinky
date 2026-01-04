#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use esp_hal::gpio::DriveMode;
use esp_hal::ledc::LSGlobalClkSource;
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::LSClockSource;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::timer::config::{Config, Duty};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default();
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let delay = esp_hal::delay::Delay::new();

    // Connect the LED to pin 7 with a 220ohm resistor
    let led = peripherals.GPIO7;
    let mut ledc = esp_hal::ledc::Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut timer = ledc.timer::<esp_hal::ledc::LowSpeed>(esp_hal::ledc::timer::Number::Timer0);

    timer
        .configure(Config {
            duty: Duty::Duty14Bit,
            clock_source: LSClockSource::APBClk,
            frequency: Rate::from_khz(1u32),
        })
        .unwrap();
    let mut channel = ledc.channel(esp_hal::ledc::channel::Number::Channel0, led);
    channel
        .configure(esp_hal::ledc::channel::config::Config {
            timer: &timer,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    // TODO: Spawn some tasks
    let _ = spawner;

    let max_duty = 100u8;
    let min_duty = 0u8;

    loop {
        for duty in min_duty..max_duty {
            channel.set_duty(duty).unwrap();
            delay.delay_millis(10);
        }

        for duty in (min_duty..max_duty).rev() {
            channel.set_duty(duty).unwrap();
            delay.delay_millis(10);
        }
    }
}
