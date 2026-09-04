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
use embedded_graphics::prelude::*;
use esp_backtrace as _;
use esp_hal::spi::master::Spi;
use esp_hal::{clock::CpuClock, ram, rng::Rng, rtc_cntl::Rtc, timer::timg::TimerGroup};
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    interrupt::software::SoftwareInterruptControl,
    spi,
    time::Rate,
};
use esp_println::println;
use log::error;
use xiao_epaper::{GreyUc8179, Uc8179};

use core::net::{IpAddr, Ipv4Addr, SocketAddr};

use embassy_net::{
    Runner, StackResources,
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_radio::wifi::{
    Config, ControllerConfig, Interface, WifiController, scan::ScanConfig, sta::StationConfig,
};
use log::info;
use sntpc::{NtpContext, NtpTimestampGenerator, get_time};
use sntpc_net_embassy::UdpSocketWrapper;

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write($val);
        x
    }};
}

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

use esp_hal::{Blocking, delay};

#[embassy_executor::task]
async fn epaper_loop(
    mut display: Uc8179<
        Spi<'static, Blocking>,
        Output<'static>,
        Output<'static>,
        Output<'static>,
        Input<'static>,
        embassy_time::Delay,
    >,
) {
    loop {
        demos::parley_demo(&mut display);
        display.flush().unwrap();
        Timer::after_millis(5000).await;
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}

const SSID: &str = "MiWiFi";
const PASSWORD: &str = "betterbread936";
const TIMEZONE: jiff::tz::TimeZone = jiff::tz::get!("America/Chicago");

#[allow(clippy::large_stack_frames)]
#[allow(unused_variables)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // Hardware Initialization
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_240MHz);
    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);
    let delay = esp_hal::delay::Delay::new();
    let rng = Rng::new();
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_int: SoftwareInterruptControl<'static> =
        SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_alloc::heap_allocator!(size: 175 * 1024);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);
    esp_println::logger::init_logger(log::LevelFilter::Info);

    // Network (NTP) Initialization
    let (mut controller, interfaces) = esp_radio::wifi::new(
        peripherals.WIFI,
        ControllerConfig::default().with_initial_config(Config::Station(
            StationConfig::default()
                .with_ssid(SSID)
                .with_password(PASSWORD.into()),
        )),
    )
    .unwrap();
    match controller.connect_async().await {
        Ok(_) => {
            let (stack, runner) = embassy_net::new(
                interfaces.station,
                embassy_net::Config::dhcpv4(Default::default()),
                mk_static!(StackResources<3>, StackResources::<3>::new()),
                (rng.random() as u64) << 32 | rng.random() as u64,
            );

            spawner.spawn(net_task(runner).unwrap());
            stack.wait_config_up().await;

            let mut rx_meta = [PacketMetadata::EMPTY; 16];
            let mut rx_buffer = [0; 1024];
            let mut tx_meta = [PacketMetadata::EMPTY; 16];
            let mut tx_buffer = [0; 1024];

            let mut socket = UdpSocket::new(
                stack,
                &mut rx_meta,
                &mut rx_buffer,
                &mut tx_meta,
                &mut tx_buffer,
            );

            socket.bind(123).unwrap();

            let socket = UdpSocketWrapper::new(socket);

            info!("Getting NTP");
            let addr: IpAddr = IpAddr::V4(Ipv4Addr::new(129, 6, 15, 30));
            let time = get_time(
                SocketAddr::from((addr, 123)),
                &socket,
                NtpContext::new(sntpc_time_embassy::EmbassyTimestampGenerator::default()),
            )
            .await
            .unwrap();
            info!("Got NTP");

            rtc.set_current_time_us(
                (time.sec() as u64 * 1_000_000) + ((time.sec_fraction() as u64 * 1_000_000) >> 32),
            );
        }
        Err(e) => {
            error!("Wifi is unavailable/bonkers: {}", e)
        }
    }

    let now = jiff::Timestamp::from_microsecond(rtc.current_time_us() as i64)
        .unwrap()
        .to_zoned(TIMEZONE);
    info!("The current time is: {now}");

    // Task Initialization
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

    let mut display = Uc8179::new(
        Spi::new(
            peripherals.SPI2,
            esp_hal::spi::master::Config::default()
                .with_frequency(Rate::from_hz(10000000))
                .with_read_bit_order(spi::BitOrder::MsbFirst)
                .with_write_bit_order(spi::BitOrder::MsbFirst)
                .with_mode(esp_hal::spi::Mode::_0),
        )
        .expect("static spi config is valid")
        .with_sck(peripherals.GPIO7)
        .with_mosi(peripherals.GPIO9),
        Output::new(peripherals.GPIO44, Level::High, OutputConfig::default()),
        Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO38, Level::High, OutputConfig::default()),
        Input::new(
            peripherals.GPIO4,
            InputConfig::default().with_pull(esp_hal::gpio::Pull::None),
        ),
        embassy_time::Delay,
    );

    display.init().unwrap();
    display.clear_white();
    display.flush().unwrap();
    delay.delay_millis(1000);
    demos::contentdisplay_demo(&mut display);
    display.flush().unwrap();
    delay.delay_millis(1000);

    spawner.spawn(epaper_loop(display).unwrap());

    loop {
        Timer::after(Duration::from_secs(10)).await;
    }
}
