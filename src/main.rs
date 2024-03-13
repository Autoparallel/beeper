#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(generic_arg_infer)]
#![feature(allocator_api)]

use core::mem::MaybeUninit;
extern crate alloc;
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either3};
use embassy_time::Timer;
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
use esp_alloc::EspHeap;
use esp_backtrace as _;
use esp_println::println;

#[global_allocator]
static ALLOCATOR: EspHeap = EspHeap::empty();

enum Color {
    Green,
    Blue,
    White,
}

struct InputButtons {
    green: GpioPin<Input<PullDown>, 4>,
    blue: GpioPin<Input<PullDown>, 1>,
    white: GpioPin<Input<PullDown>, 10>,
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
        green: io.pins.gpio4.into_pull_down_input(),
        blue: io.pins.gpio1.into_pull_down_input(),
        white: io.pins.gpio10.into_pull_down_input(),
    };

    // Set up a blinky task
    let blinky_led = io.pins.gpio0.into_push_pull_output();
    spawner.spawn(blinky(blinky_led)).unwrap();

    // The main loop
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
    unsafe { ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, 1024) };
    // let mut message = Vec::with_capacity_in(1024, ALLOCATOR);
    let mut message = Vec::new();
    loop {
        println!("Waiting for button press...");
        let color = input.get_input().await;
        Timer::after_millis(300).await;
        println!("Button pressed!");
        match color {
            Color::Green => {
                println!("Green button pressed!");
                println!("Message: {:?}", message);
                message.clear();
                println!("Message cleared.")
            }
            Color::Blue => {
                println!("Blue button pressed!");
                message.push(true)
            }
            Color::White => {
                println!("White button pressed!");
                message.push(false);
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
