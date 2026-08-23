// The Display Code
use crate::{drawPixel, BLACK, WHITE};
use embedded_graphics::{
    pixelcolor::{self, BinaryColor, Gray4},
    prelude::*,
};

pub struct Display;
pub struct FourGrayDisplay;

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(800, 480)
    }
}

impl OriginDimensions for FourGrayDisplay {
    fn size(&self) -> Size {
        Size::new(800, 480)
    }
}

const fn bincolor_into(b: BinaryColor) -> u32 {
    match b {
        BinaryColor::On => BLACK,
        BinaryColor::Off => WHITE,
    }
}

fn graycolor_into(b: Gray4) -> u32 {
    let rgb565: pixelcolor::Rgb565 = b.into();
    let u16ver: pixelcolor::raw::RawU16 = rgb565.into();
    u16ver.into_inner() as u32
}

impl DrawTarget for Display {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            unsafe {
                drawPixel(coord.x, coord.y, bincolor_into(color));
            }
        }

        Ok(())
    }
}

impl DrawTarget for FourGrayDisplay {
    type Color = Gray4;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            unsafe {
                drawPixel(coord.x, coord.y, graycolor_into(color));
            }
        }

        Ok(())
    }
}

// The Serial Port Code
