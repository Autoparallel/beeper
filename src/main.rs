#![no_std]
#![no_main]

use esp32c6_hal::{
    clock::ClockControl,
    gpio::{Gpio0, GpioPin},
    peripherals::Peripherals,
    prelude::*,
    Delay, IO,
};
use esp_backtrace as _;
use esp_println::println;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = io.pins.gpio0.into_push_pull_output();

    println!("Hello world!");
    loop {
        println!("Loop...");
        led.set_high().unwrap();
        delay.delay_ms(1000u32);
        led.set_low().unwrap();
        delay.delay_ms(1000u32);
    }
}
