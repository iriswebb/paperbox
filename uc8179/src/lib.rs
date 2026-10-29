//! minimal UC8179 (GD7965) driver for the seeed studio XIAO 7.5" ePaper panel.
//!
//! command sequence ported from bb_epaper (EP75_800x480) as used in the
//! Seeed TRMNL firmware for this exact board.

#![no_std]

use alloc::vec;
use alloc::vec::Vec;
use embedded_graphics::pixelcolor::{BinaryColor, Gray2};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiBus;

extern crate alloc;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 480;
const ROW_BYTES: usize = (WIDTH as usize) / 8;
pub const FB_SIZE: usize = ROW_BYTES * HEIGHT as usize;

/// The UC8179 Command Table, from https://www.buydisplay.com/download/ic/UC8179.pdf, page 8 to 11
#[allow(non_upper_case_globals)]
pub mod command {
    pub const PanelSetting: u8 = 0x00;
    pub const PowerSetting: u8 = 0x01;
    pub const PowerOff: u8 = 0x02;
    pub const PowerOffSequenceSetting: u8 = 0x03;
    pub const PowerOn: u8 = 0x04;
    pub const PowerOnMeasure: u8 = 0x05;
    pub const BoosterSoftStart: u8 = 0x06;
    pub const DeepSleep: u8 = 0x07;

    pub const DataTransmission1: u8 = 0x10;
    pub const DataStop: u8 = 0x11;
    pub const DisplayRefresh: u8 = 0x12;
    pub const DataTransmission2: u8 = 0x13;
    pub const DualSPI: u8 = 0x15;
    pub const AutoSequence: u8 = 0x17;

    pub const LutVCOM: u8 = 0x20;
    pub const LutWW: u8 = 0x21;
    pub const LutKW: u8 = 0x22;
    pub const LutWK: u8 = 0x23;
    pub const LutKK: u8 = 0x24;
    pub const LutBorder: u8 = 0x25;
    pub const LutOption: u8 = 0x2A;
    pub const LutOptionKW: u8 = 0x2B;

    pub const PLL: u8 = 0x30;
    pub const TemperatureCalibration: u8 = 0x40;
    pub const TemperatureSelection: u8 = 0x41;
    pub const TemperatureWrite: u8 = 0x42;
    pub const TemperatureRead: u8 = 0x43;
    pub const PanelBreakCheck: u8 = 0x44;

    pub const CDI: u8 = 0x50;
    pub const LowPower: u8 = 0x51;
    pub const EndVoltageSetting: u8 = 0x52;

    pub const TCON: u8 = 0x60;
    pub const ResolutionSetting: u8 = 0x61;
    pub const SourceStartSetting: u8 = 0x65;

    pub const Revision: u8 = 0x70;
    pub const GetStatus: u8 = 0x71;
    pub const AutoVCOM: u8 = 0x80;
    pub const ReadVCOM: u8 = 0x81;
    pub const VCOMDCSetting: u8 = 0x82;

    pub const PartialWindow: u8 = 0x90;
    pub const PartialIn: u8 = 0x91;
    pub const PartialOut: u8 = 0x92;

    pub const ProgramMode: u8 = 0xA0;
    pub const ActiveProgramming: u8 = 0xA1;
    pub const ReadOTP: u8 = 0xA2;
    pub const Cascade: u8 = 0xE0;
    pub const PowerSaving: u8 = 0xE3;
    pub const LVDSelect: u8 = 0xE4;
    pub const ForceTempSet: u8 = 0xE5;
    pub const TempBoundPhaseC3: u8 = 0xE7;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error<S, P> {
    Spi(S),
    Pin(P),
    TimedOut,
}

pub struct Uc8179<SPI, CS, DC, RST, BUSY, DELAY> {
    spi: SPI,
    cs: CS,
    dc: DC,
    rst: RST,
    busy: BUSY,
    delay: DELAY,
    fb: Vec<u8>,
    fb2: Vec<u8>,
}

pub struct GreyUc8179<'a, SPI, CS, DC, RST, BUSY, DELAY>(
    pub &'a mut Uc8179<SPI, CS, DC, RST, BUSY, DELAY>,
);

impl<SPI, CS, DC, RST, BUSY, DELAY> Uc8179<SPI, CS, DC, RST, BUSY, DELAY>
where
    SPI: SpiBus,
    CS: OutputPin,
    DC: OutputPin<Error = CS::Error>,
    RST: OutputPin<Error = CS::Error>,
    BUSY: InputPin<Error = CS::Error>,
    DELAY: DelayNs,
{
    pub fn new(spi: SPI, cs: CS, dc: DC, rst: RST, busy: BUSY, delay: DELAY) -> Self {
        Self {
            spi,
            cs,
            dc,
            rst,
            busy,
            delay,
            fb: vec![0x00; 48000],
            fb2: Vec::new(),
        }
    }

    fn cmd(&mut self, c: u8) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.dc.set_low().map_err(Error::Pin)?;
        self.cs.set_low().map_err(Error::Pin)?;
        self.spi.write(&[c]).map_err(Error::Spi)?;
        self.spi.flush().map_err(Error::Spi)?;
        self.cs.set_high().map_err(Error::Pin)
    }

    fn data(&mut self, d: &[u8]) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.dc.set_high().map_err(Error::Pin)?;
        self.cs.set_low().map_err(Error::Pin)?;
        self.spi.write(d).map_err(Error::Spi)?;
        self.spi.flush().map_err(Error::Spi)?;
        self.cs.set_high().map_err(Error::Pin)
    }

    fn cmd_with(&mut self, c: u8, d: &[u8]) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.cmd(c)?;
        self.data(d)
    }

    // UC81xx busy: LOW = busy, HIGH = ready (idle).
    fn wait_busy(&mut self, timeout_ms: u32) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.delay.delay_ms(10);
        let mut elapsed = 10u32;

        while self.busy.is_low().unwrap_or(true) {
            self.delay.delay_ms(100);
            elapsed += 100;
            if elapsed >= timeout_ms {
                return Err(Error::TimedOut);
            }
        }
        Ok(())
    }

    fn cmd_with_data_wait(&mut self, c: u8, d: &[u8]) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.cmd(c)?;
        self.data(d)?;
        self.wait_busy(6000)
    }

    fn cmd_with_wait(&mut self, c: u8) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.cmd(c)?;
        self.wait_busy(6000)
    }

    fn send_luts(
        &mut self,
        vcom: &[u8; 42],
        w2w: &[u8; 42],
        k2w: &[u8; 42],
        w2k: &[u8; 42],
        k2k: &[u8; 42],
    ) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.wait_busy(3000)?;
        self.cmd_with_data_wait(command::LutVCOM, vcom)?;
        self.cmd_with_data_wait(command::LutWW, w2w)?;
        self.cmd_with_data_wait(command::LutKW, k2w)?;
        self.cmd_with_data_wait(command::LutWK, w2k)?;
        self.cmd_with_data_wait(command::LutKK, k2k)
    }

    fn reset(&mut self) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.rst.set_low().map_err(Error::Pin)?;
        self.delay.delay_ms(10);
        self.rst.set_high().map_err(Error::Pin)?;
        self.delay.delay_ms(20);
        self.wait_busy(5000)
    }

    // bb_epaper epd75_init_sequence_full: sent before every refresh.
    //   PWR → PON → BUSY_WAIT → PSR → TRES → 0x15 → CDI → TCON
    pub fn init(&mut self) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.fb2 = Vec::new();
        pub const MONOCRHOME_BLACK_LUT: [u8; 42] = [
            0xAA, 0x06, 0x0C, 0x17, 0x02, 0x01, 0x15, 0x02, 0x1C, 0x02, 0x0D, 0x01, 0x40, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        pub const MONOCRHOME_WHITE_LUT: [u8; 42] = [
            0x55, 0x06, 0x0C, 0x17, 0x02, 0x01, 0x2A, 0x02, 0x1C, 0x02, 0x0D, 0x01, 0x80, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        pub const MONOCHROME_VCOM_LUT: [u8; 42] = [
            0x26, 0x0F, 0x18, 0x18, 0x14, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        self.reset()?;
        self.cmd_with(command::PowerSetting, &[0x07, 0x07, 0x3f, 0x3f, 0x09])?;
        // self.cmd_with(command::PLL, &[0x06])?;
        // self.cmd_with(command::VCOMDCSetting, &[0x16])?;
        self.cmd_with(command::BoosterSoftStart, &[0x17, 0x17, 0x28, 0x17])?;
        self.cmd_with_wait(command::PowerOn)?;
        self.cmd_with(command::PanelSetting, &[0x1f])?;
        self.cmd_with(command::ResolutionSetting, &[0x03, 0x20, 0x01, 0xE0])?;
        self.cmd_with(command::DualSPI, &[0x00])?;
        self.cmd_with(command::CDI, &[0x10, 0x07])?; // &[0x10, 0x07] ?
        self.cmd_with(command::TCON, &[0x22])?;
        self.send_luts(
            &MONOCHROME_VCOM_LUT,
            &MONOCRHOME_WHITE_LUT,
            &MONOCRHOME_WHITE_LUT,
            &MONOCRHOME_BLACK_LUT,
            &MONOCRHOME_BLACK_LUT,
        )
    }

    pub fn init_gray(&mut self) -> Result<(), Error<SPI::Error, CS::Error>> {
        pub const GRAY_VCOM_LUT: [u8; 42] = [
            0x00, 0x00, 0x06, 0x08, 0x07, 0x01, 0x00, 0x06, 0x0A, 0x0B, 0x0A, 0x01, 0x00, 0x03,
            0x03, 0x00, 0x00, 0x03, 0x00, 0x05, 0x09, 0x06, 0x06, 0x01, 0x00, 0x02, 0x02, 0x0A,
            0x0A, 0x01, 0x00, 0x0A, 0x11, 0x06, 0x07, 0x01, 0x00, 0x02, 0x01, 0x02, 0x01, 0x01,
        ];
        pub const GRAY_W2W_LUT: [u8; 42] = [
            0x15, 0x00, 0x06, 0x08, 0x07, 0x01, 0x54, 0x06, 0x0A, 0x0B, 0x0A, 0x01, 0x90, 0x03,
            0x03, 0x00, 0x00, 0x03, 0x2A, 0x05, 0x09, 0x06, 0x06, 0x01, 0xAA, 0x02, 0x02, 0x0A,
            0x0A, 0x01, 0x00, 0x0A, 0x11, 0x06, 0x07, 0x01, 0x28, 0x02, 0x01, 0x02, 0x01, 0x01,
        ];
        pub const GRAY_K2W_LUT: [u8; 42] = [
            0x2A, 0x00, 0x06, 0x08, 0x07, 0x01, 0x59, 0x06, 0x0A, 0x0B, 0x0A, 0x01, 0x90, 0x03,
            0x03, 0x00, 0x00, 0x03, 0x5A, 0x05, 0x09, 0x06, 0x06, 0x01, 0xA8, 0x02, 0x02, 0x0A,
            0x0A, 0x01, 0x45, 0x0A, 0x11, 0x06, 0x07, 0x01, 0xA8, 0x02, 0x01, 0x02, 0x01, 0x01,
        ];
        pub const GRAY_W2K_LUT: [u8; 42] = [
            0x16, 0x00, 0x06, 0x08, 0x07, 0x01, 0xA0, 0x06, 0x0A, 0x0B, 0x0A, 0x01, 0x90, 0x03,
            0x03, 0x00, 0x00, 0x03, 0x99, 0x05, 0x09, 0x06, 0x06, 0x01, 0xA0, 0x02, 0x02, 0x0A,
            0x0A, 0x01, 0x40, 0x0A, 0x11, 0x06, 0x07, 0x01, 0x20, 0x02, 0x01, 0x02, 0x01, 0x01,
        ];
        pub const GRAY_K2K_LUT: [u8; 42] = [
            0x26, 0x00, 0x06, 0x08, 0x07, 0x01, 0x6A, 0x06, 0x0A, 0x0B, 0x0A, 0x01, 0x90, 0x03,
            0x03, 0x00, 0x00, 0x03, 0x65, 0x05, 0x09, 0x06, 0x06, 0x01, 0x50, 0x02, 0x02, 0x0A,
            0x0A, 0x01, 0x10, 0x0A, 0x11, 0x06, 0x07, 0x01, 0x10, 0x02, 0x01, 0x02, 0x01, 0x01,
        ];

        self.fb2.resize(48000, 0x00);
        self.reset()?;
        self.cmd_with(command::PowerSetting, &[0x07, 0x17, 0x3f, 0x3f, 0x07])?;
        self.cmd_with(command::PLL, &[0x06])?;
        self.cmd_with(command::VCOMDCSetting, &[0x12])?;
        self.cmd_with(command::BoosterSoftStart, &[0x27, 0x27, 0x28, 0x17])?;
        self.cmd_with_wait(command::PowerOn)?;
        self.cmd_with(command::PanelSetting, &[0x3f])?;
        self.cmd_with(command::ResolutionSetting, &[0x03, 0x20, 0x01, 0xE0])?;
        // self.cmd_with(command::DualSPI, &[0x00])?;
        self.cmd_with(command::CDI, &[0x10, 0x07])?; // &[0x10, 0x07] ?
        // self.cmd_with(command::TCON, &[0x22])?;
        self.send_luts(
            &GRAY_VCOM_LUT,
            &GRAY_W2W_LUT,
            &GRAY_K2W_LUT,
            &GRAY_W2K_LUT,
            &GRAY_K2K_LUT,
        )
    }

    pub fn sleep(&mut self) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.cmd_with(command::CDI, &[0xf7])?;
        self.cmd_with_wait(command::PowerOff)?;
        self.cmd_with(command::DeepSleep, &[0xA5])
    }

    pub fn clear_white(&mut self) {
        self.fb.fill(0x00);

        if !self.fb2.is_empty() {
            self.fb2.fill(0x00);
        }
    }

    pub fn load_raw(&mut self, data: &[u8; FB_SIZE]) {
        self.fb.copy_from_slice(data);
    }

    pub fn load_raw_fb2(&mut self, data: &[u8; FB_SIZE]) {
        self.fb2.copy_from_slice(data);
    }

    pub fn framebuffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.fb
    }

    pub fn flush(&mut self) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.write_framebuffer()?;
        self.cmd_with_wait(command::DisplayRefresh)
    }

    fn write_framebuffer(&mut self) -> Result<(), Error<SPI::Error, CS::Error>> {
        if !self.fb2.is_empty() {
            self.cmd(command::DataTransmission1)?;
            self.dc.set_high().map_err(Error::Pin)?;
            self.cs.set_low().map_err(Error::Pin)?;
            self.spi.write(&self.fb2).map_err(Error::Spi)?;
            self.spi.flush().map_err(Error::Spi)?;
            self.cs.set_high().map_err(Error::Pin)?;
        }

        self.cmd(command::DataTransmission2)?;
        self.dc.set_high().map_err(Error::Pin)?;
        self.cs.set_low().map_err(Error::Pin)?;
        self.spi.write(&self.fb).map_err(Error::Spi)?;
        self.spi.flush().map_err(Error::Spi)?;
        self.cs.set_high().map_err(Error::Pin)?;
        self.cmd(command::DataStop)
    }

    pub fn init_partial(&mut self) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.init()?;

        const LUT_PARTIAL_VCOM: [u8; 42] = [
            0x00, 30, 5, 30, 5, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const LUT_PARTIAL_WW: [u8; 42] = [
            0x00, 30, 5, 30, 5, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const LUT_PARTIAL_BW: [u8; 42] = [
            0x5A, 30, 5, 30, 5, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const LUT_PARTIAL_WB: [u8; 42] = [
            0x84, 30, 5, 30, 5, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const LUT_PARTIAL_BB: [u8; 42] = [
            0x00, 30, 5, 30, 5, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const LUT_PARTIAL_BD: [u8; 42] = [
            0x00, 30, 5, 30, 5, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        // override to register LUT mode (bit 5 = 1)
        self.cmd_with(command::PanelSetting, &[0x3f])?;
        self.cmd_with(command::VCOMDCSetting, &[0x26])?;
        // N2OCP=1 (bit 4) for new→old RAM copy, BDV=11 for LUTBD border
        self.cmd_with(command::CDI, &[0x39, 0x07])?;

        self.send_luts(
            &LUT_PARTIAL_VCOM,
            &LUT_PARTIAL_WW,
            &LUT_PARTIAL_BW,
            &LUT_PARTIAL_WB,
            &LUT_PARTIAL_BB,
        )?;
        self.cmd_with_data_wait(command::LutBorder, &LUT_PARTIAL_BD)
    }

    // uses register LUTs with a partial waveform — only changed pixels
    // are driven. may ghost over time; use flush() periodically.
    pub fn flush_partial(&mut self, area: Rectangle) -> Result<(), Error<SPI::Error, CS::Error>> {
        self.init_partial()?;
        self.write_framebuffer()?;

        let area = area.intersection(&self.bounding_box());
        let le_seq = [
            area.top_left.x,
            area.bottom_right().unwrap_or_default().x - 1,
            area.top_left.y,
            area.bottom_right().unwrap_or_default().y - 1,
        ]
        .map(|a| a.to_le_bytes());
        #[rustfmt::skip]
        self.cmd_with(command::PartialWindow, le_seq.as_flattened())?;
        self.data(&[0x01])?;

        self.cmd_with_wait(command::DisplayRefresh)
    }

    fn set_pixel(&mut self, x: u32, y: u32, black: bool) {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }
        let idx = y as usize * ROW_BYTES + (x as usize >> 3);
        let mask = 0x80u8 >> (x & 7);
        if black {
            self.fb[idx] &= !mask;
        } else {
            self.fb[idx] |= mask;
        }
    }

    fn set_greypixel(&mut self, x: u32, y: u32, color: u8) {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }
        let idx = y as usize * ROW_BYTES + (x as usize >> 3);
        let mask = 0x80u8 >> (x & 7);
        let (new, old) = match color {
            0b00 => (true, true),
            0b01 => (false, true),
            0b10 => (true, false),
            0b11 => (false, false),
            _ => panic!("Grey2 Pixel more than 0b11"),
        };

        if new {
            self.fb[idx] |= mask;
        } else {
            self.fb[idx] &= !mask;
        }

        if old {
            self.fb2[idx] |= mask;
        } else {
            self.fb2[idx] &= !mask;
        }
    }
}

impl<SPI, CS, DC, RST, BUSY, DELAY> OriginDimensions for Uc8179<SPI, CS, DC, RST, BUSY, DELAY> {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl<SPI, CS, DC, RST, BUSY, DELAY> DrawTarget for Uc8179<SPI, CS, DC, RST, BUSY, DELAY>
where
    SPI: SpiBus,
    CS: OutputPin,
    DC: OutputPin<Error = CS::Error>,
    RST: OutputPin<Error = CS::Error>,
    BUSY: InputPin<Error = CS::Error>,
    DELAY: DelayNs,
{
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.y >= 0 {
                self.set_pixel(point.x as u32, point.y as u32, color.is_on());
            }
        }
        Ok(())
    }
}

impl<'a, SPI, CS, DC, RST, BUSY, DELAY> OriginDimensions
    for GreyUc8179<'a, SPI, CS, DC, RST, BUSY, DELAY>
{
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl<'a, SPI, CS, DC, RST, BUSY, DELAY> DrawTarget for GreyUc8179<'a, SPI, CS, DC, RST, BUSY, DELAY>
where
    SPI: SpiBus,
    CS: OutputPin,
    DC: OutputPin<Error = CS::Error>,
    RST: OutputPin<Error = CS::Error>,
    BUSY: InputPin<Error = CS::Error>,
    DELAY: DelayNs,
{
    type Color = Gray2;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.y >= 0 {
                self.0
                    .set_greypixel(point.x as u32, point.y as u32, color.luma());
            }
        }
        Ok(())
    }
}
