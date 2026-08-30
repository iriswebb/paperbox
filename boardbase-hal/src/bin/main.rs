#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
#![allow(unused_imports)]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_graphics::prelude::*;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::spi::master::Spi;
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    interrupt::software::SoftwareInterruptControl,
    spi,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;
use xiao_epaper::{GreyUc8179, Uc8179};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn blink(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::task]
async fn meow() {
    let mut i = 0;
    loop {
        println!("meow {i}");
        Timer::after_millis(500).await;
        i += 1;
    }
}

#[allow(clippy::large_stack_frames)]
#[allow(unused_variables)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_240MHz);
    let peripherals = esp_hal::init(config);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let led = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());
    spawner.spawn(blink(led).unwrap());
    spawner.spawn(meow().unwrap());

    let button0 = Input::new(
        peripherals.GPIO2,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::None),
    );
    let button1 = Input::new(
        peripherals.GPIO3,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::None),
    );
    let button2 = Input::new(
        peripherals.GPIO5,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::None),
    );

    let spi = Spi::new(
        peripherals.SPI2,
        esp_hal::spi::master::Config::default()
            .with_frequency(Rate::from_hz(10000000))
            .with_read_bit_order(spi::BitOrder::MsbFirst)
            .with_write_bit_order(spi::BitOrder::MsbFirst)
            .with_mode(esp_hal::spi::Mode::_0),
    )
    .expect("static spi config is valid")
    .with_sck(peripherals.GPIO7)
    .with_mosi(peripherals.GPIO9);

    let cs = Output::new(peripherals.GPIO44, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO38, Level::High, OutputConfig::default());
    let busy = Input::new(
        peripherals.GPIO4,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::None),
    );
    let delay = esp_hal::delay::Delay::new();

    #[allow(static_mut_refs)]
    let mut display = Uc8179::new(spi, cs, dc, rst, busy, delay);
    display.init().unwrap();
    display.clear_white();
    display.flush().unwrap();
    delay.delay_micros(1000);
    demos::parley_demo(&mut display);
    display.flush().unwrap();
    display.sleep().unwrap();

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let sw_int: SoftwareInterruptControl<'static> =
        SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    esp_println::logger::init_logger_from_env();

    let mut fc = 0;
    display.init_partial().unwrap();
    display.clear_white();
    loop {
        demos::badapple_demo(&mut display, fc);
        display.flush_partial(display.bounding_box()).unwrap();
        fc += 1;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}
