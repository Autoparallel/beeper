#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_hal_async::digital::Wait;
use esp32c6_hal::{
    clock::{ClockControl, Clocks},
    embassy::{self, executor::Executor},
    gpio::{GpioPin, Output, PushPull},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Delay, IO,
};
use esp_backtrace as _;
use esp_println::println;

#[main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    println!("Hello world!\n Welcome to this ESP32-C6 board!");

    // Set up the async stuff
    let clocks = ClockControl::max(system.clock_control).freeze();
    embassy::init(&clocks, TimerGroup::new(peripherals.TIMG0, &clocks));

    // Set up the io
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut input = io.pins.gpio1.into_pull_down_input();
    let led = io.pins.gpio0.into_push_pull_output();

    spawner.spawn(blinky(led)).unwrap();

    loop {
        println!("Waiting for button press...");
        input.wait_for_high().await.unwrap();
        Timer::after_millis(200).await;
        println!("Button pressed!");
    }
}

#[embassy_executor::task]
async fn blinky(mut led: GpioPin<Output<PushPull>, 0>) {
    println!("Blinking LED on GPIO0...");
    loop {
        led.set_high().unwrap();
        Timer::after_millis(100).await;
        led.set_low().unwrap();
        Timer::after_millis(100).await;
    }
}
