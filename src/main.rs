#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(generic_arg_infer)]

use embassy_executor::Spawner;
use embassy_futures::select::{select3, select_array, Either3};
use embassy_time::{Duration, Timer};
use embedded_hal_async::digital::Wait;
use esp32c6_hal::{
    clock::ClockControl,
    embassy::{self},
    gpio::{GpioPin, Input, Output, PullDown, PushPull},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    IO,
};
use esp_backtrace as _;
use esp_println::println;

enum Color {
    Green,
    Blue,
    White,
}

struct InputButtons {
    green: GpioPin<Input<PullDown>, 1>,
    blue: GpioPin<Input<PullDown>, 8>,
    white: GpioPin<Input<PullDown>, 2>,
}

impl InputButtons {
    async fn get_input(&mut self) -> Color {
        let either = select3(
            self.green.wait_for_high(),
            self.blue.wait_for_high(),
            self.white.wait_for_high(),
        )
        .await;
        match either {
            Either3::First(_) => Color::Green,
            Either3::Second(_) => Color::Blue,
            Either3::Third(_) => Color::White,
        }
    }
}

struct OutputLeds {
    green: GpioPin<Output<PushPull>, 18>,
    blue: GpioPin<Output<PushPull>, 19>,
    white: GpioPin<Output<PushPull>, 20>,
}

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
    let mut input = InputButtons {
        green: io.pins.gpio1.into_pull_down_input(),
        blue: io.pins.gpio8.into_pull_down_input(),
        white: io.pins.gpio2.into_pull_down_input(),
    };
    let mut output = OutputLeds {
        green: io.pins.gpio18.into_push_pull_output(),
        blue: io.pins.gpio19.into_push_pull_output(),
        white: io.pins.gpio20.into_push_pull_output(),
    };

    // Set up a blinky task
    let blinky_led = io.pins.gpio0.into_push_pull_output();
    spawner.spawn(blinky(blinky_led)).unwrap();

    // The main loop
    loop {
        println!("Waiting for button press...");
        let color = input.get_input().await;
        Timer::after_millis(200).await;
        println!("Button pressed!");
        match color {
            Color::Green => {
                println!("Green button pressed!");
                output.green.set_high().unwrap();
                Timer::after(Duration::from_secs(2)).await;
                output.green.set_low().unwrap();
            }
            Color::Blue => {
                println!("Blue button pressed!");
                output.blue.set_high().unwrap();
                Timer::after(Duration::from_secs(2)).await;
                output.blue.set_low().unwrap();
            }
            Color::White => {
                println!("White button pressed!");
                output.white.set_high().unwrap();
                Timer::after(Duration::from_secs(2)).await;
                output.white.set_low().unwrap();
            }
        }
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
