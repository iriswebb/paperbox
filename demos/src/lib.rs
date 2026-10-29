#![no_std]
#![allow(clippy::type_complexity)]
#![feature(associated_type_defaults)]

extern crate alloc;

use core::num::NonZeroU16;

use alloc::string::String;
use core_maths::CoreFloat;
use embedded_graphics::{
    geometry::Point,
    image::ImageRaw,
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::{BinaryColor, Gray2},
    prelude::*,
    primitives::{Circle, CornerRadii, PrimitiveStyle, PrimitiveStyleBuilder, RoundedRectangle},
    text::{Text, renderer::TextRenderer},
};
use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    spi::SpiBus,
};
use xiao_epaper::{GreyUc8179, Uc8179};

use embedded_badapple::DrawInto;

pub mod badapple;
pub mod shaped_text_demo;

/// Prevents every demo from having one million generic arguments
pub trait Driver {
    type SPI: SpiBus;
    type E;
    type CS: OutputPin<Error = Self::E>;
    type DC: OutputPin<Error = Self::E>;
    type RST: OutputPin<Error = Self::E>;
    type BUSY: InputPin<Error = Self::E>;
    type Delay: DelayNs;

    type FINAL = Uc8179<Self::SPI, Self::CS, Self::DC, Self::RST, Self::BUSY, Self::Delay>;

    fn inner_mut(
        &mut self,
    ) -> &mut Uc8179<Self::SPI, Self::CS, Self::DC, Self::RST, Self::BUSY, Self::Delay>;
}

impl<
    SPI: SpiBus,
    CS: OutputPin,
    DC: OutputPin<Error = CS::Error>,
    RST: OutputPin<Error = CS::Error>,
    BUSY: InputPin<Error = CS::Error>,
    DELAY: DelayNs,
> Driver for Uc8179<SPI, CS, DC, RST, BUSY, DELAY>
{
    type SPI = SPI;
    type BUSY = BUSY;
    type E = CS::Error;
    type CS = CS;
    type DC = DC;
    type RST = RST;
    type Delay = DELAY;

    fn inner_mut(
        &mut self,
    ) -> &mut Uc8179<Self::SPI, Self::CS, Self::DC, Self::RST, Self::BUSY, Self::Delay> {
        self
    }
}

pub fn mandlebrot(real: f32, imag: f32) -> bool {
    let mut x = 0f32;
    let mut y = 0f32;
    for _i in 0..ITERLIMIT {
        if (x * x + y * y) >= 4.0 {
            return false;
        }
        let xtemp = (x * x) - (y * y) + real;
        y = 2.0 * x * y + imag;
        x = xtemp;
    }
    true
}
const ITERLIMIT: u32 = 10;
const MANDLERANGE_X: (f32, f32) = (-2.5, 0.5);
const MANDLERANGE_Y: (f32, f32) = (-1.2, 1.2);

/// Transposes a range (x..y..step) onto a range (a..b), returning the step required for the second range
///
/// Ex:
///     (0..100..1) transposed onto (-1.0..1.0) has a step of 0.02, so the correct range is (-1.0..1.0..0.02)
///
/// Useful in scaling, transformation, and for floating point functions
pub fn range_transpose(x: f32, y: f32, step: f32, a: f32, b: f32) -> f32 {
    (b - a) / ((y - x) / step).ceil()
}

pub fn mandlebrot_demo<D: Driver>(driver: &mut D) {
    let driver = driver.inner_mut();
    let mandle_step_x = range_transpose(0f32, 800f32, 1f32, MANDLERANGE_X.0, MANDLERANGE_X.1);
    let mandle_step_y = range_transpose(0f32, 480f32, 1f32, MANDLERANGE_Y.0, MANDLERANGE_Y.1);
    for y in 0..480 {
        for x in 0..800 {
            let m = mandlebrot(
                MANDLERANGE_X.0 + (mandle_step_x * x as f32),
                MANDLERANGE_Y.0 + (mandle_step_y * y as f32),
            );
            embedded_graphics::Pixel(Point::new(x, y), BinaryColor::from(m)).draw(driver);
        }
    }
}

static mut ZLIB_DECOMP_BUF: [u8; 48000] = [0; 48000];
pub const PACKED_IMAGE_ZLIB: &[u8] = include_bytes!("packed_image_data/o4x4_3.packed.zlib");

/// :D
/// # Safety
///
/// This function should be safe, and is unsafe only due to externed C functions
#[allow(static_mut_refs)]
pub fn zlib_demo<D: Driver>(driver: &mut D) {
    use zlib_rs::decompress_slice;
    let driver = driver.inner_mut();
    let (_decomp, rc) = unsafe {
        decompress_slice(
            &mut ZLIB_DECOMP_BUF,
            PACKED_IMAGE_ZLIB,
            zlib_rs::InflateConfig::default(),
        )
    };
    let data = unsafe { ImageRaw::<BinaryColor>::new(&ZLIB_DECOMP_BUF, 800) };
    assert_eq!(rc, zlib_rs::ReturnCode::Ok);
    data.draw(driver);
}

pub fn badapple_demo<D: Driver>(driver: &mut D, fc: usize) {
    let driver = driver.inner_mut();
    ImageRaw::<BinaryColor>::new(badapple::BADAPPLE[fc % badapple::BADAPPLE.len()], 480)
        .draw(driver);
}

#[derive(Debug, Clone)]
pub struct PlainTextDemo<C: PixelColor, T: TextRenderer<Color = C>> {
    pub text: String,
    pub background: Background<C>,
    pub text_style: T,
}

#[derive(Debug, Clone)]
pub struct Background<C: PixelColor> {
    style: PrimitiveStyle<C>,
    radii: Option<CornerRadii>,
}

impl<C: PixelColor, T: TextRenderer<Color = C> + Clone, D: DrawTarget<Color = C>> DrawInto<D>
    for PlainTextDemo<C, T>
{
    fn draw_into(
        &self,
        t: &mut D,
        r: embedded_graphics::primitives::Rectangle,
    ) -> Result<(), D::Error> {
        if let Some(rad) = self.background.radii {
            RoundedRectangle::new(r, rad)
                .into_styled(self.background.style)
                .draw(t)?;
        } else {
            r.into_styled(self.background.style).draw(t)?;
        }

        #[allow(unused_must_use)]
        Text::with_alignment(
            &self.text,
            r.center(),
            self.text_style.clone(),
            embedded_graphics::text::Alignment::Center,
        )
        .draw(t);

        Ok(())
    }
}

/// # Safety
///
/// This function should be safe, and is unsafe only due to externed C functions
pub fn contentdisplay_demo<D: Driver>(driver: &mut D)
where
    <D as Driver>::Delay: 'static,
    <D as Driver>::BUSY: 'static,
    <D as Driver>::RST: 'static,
    <D as Driver>::CS: 'static,
    <D as Driver>::DC: 'static,
    <D as Driver>::SPI: 'static,
{
    let disp = driver.inner_mut();
    use alloc::boxed::Box;
    use alloc::vec;
    use embedded_badapple::demos::split::Split;

    let make_text = |s| {
        Box::new(PlainTextDemo {
            text: String::from(s),
            background: Background {
                style: PrimitiveStyleBuilder::new()
                    .fill_color(BinaryColor::On)
                    .build(),
                radii: None,
            },
            text_style: MonoTextStyle::new(&FONT_10X20, BinaryColor::Off),
        })
    };

    let w = make_text("meow");
    let v = Split {
        is_horiz: false,
        widgets: vec![
            (NonZeroU16::MIN, make_text("I'm a cat")),
            (NonZeroU16::MIN, make_text("meow")),
        ],
    };

    let s = Split {
        is_horiz: true,
        widgets: vec![
            (NonZeroU16::new(3).unwrap(), Box::new(v)),
            (NonZeroU16::new(2).unwrap(), make_text("mrrp")),
        ],
    };

    s.draw_into(disp, disp.bounding_box());
}

pub fn basegrey_demo<D: Driver>(driver: &mut D) {
    let driver = driver.inner_mut();
    let mut gray = GreyUc8179(driver);
    let crc = Circle::with_center(gray.bounding_box().center(), 100).into_styled(
        PrimitiveStyleBuilder::new()
            .fill_color(Gray2::new(3))
            .build(),
    );
    crc.draw(&mut gray);
}

const TTF_FONT_DATA: &[u8] = include_bytes!("fonts/LiberationSerif-Italic.ttf") as &[u8];

pub fn femtofont_demo<D: Driver>(driver: &mut D) {
    use embedded_graphics::{
        pixelcolor::Gray2,
        prelude::*,
        text::{Alignment, Text},
    };
    let driver = driver.inner_mut();
    let mut gray = GreyUc8179(driver);

    use alloc::sync::Arc;

    let f = Arc::new(ttf_parser::Face::parse(TTF_FONT_DATA, 0).unwrap());
    let font = femtofont::Font::from_face(f, femtofont::FontSettings::default()).unwrap();
    // Specify color and location
    let style =
        eg_femtofont::FemtoFontTextStyle::with_aa_color(&font, Gray2::BLACK, Gray2::WHITE, 140);
    let rendered_text =
        Text::with_alignment("FᴇᴍᴛᴏFᴏɴᴛ", Point::new(400, 240), style, Alignment::Center);

    // Render
    rendered_text.draw(&mut gray).unwrap();
}

pub fn parley_demo<D: Driver>(driver: &mut D) {
    let driver = driver.inner_mut();
    use embedded_graphics::pixelcolor::BinaryColor;

    shaped_text_demo::draw_shaped_text(
        driver,
        "Shaped Embedded Graphics\n\nالرسومات المدمجة",
        include_bytes!("fonts/Rubik-Bold.ttf"),
        64.0,
        Some(790.0),
        BinaryColor::Off,
        BinaryColor::On,
        30,
        1.3,
        Point::zero(),
        parley::Alignment::Center,
    );
}

pub fn parley_grey_demo<D: Driver>(driver: &mut D) {
    let driver = driver.inner_mut();
    let mut gray = GreyUc8179(driver);

    shaped_text_demo::draw_shaped_text(
        &mut gray,
        "Shaped Embedded Graphics\n\nالرسومات المدمجة",
        include_bytes!("fonts/Rubik-Bold.ttf"),
        64.0,
        Some(790.0),
        Gray2::BLACK,
        Gray2::WHITE,
        30,
        1.3,
        Point::zero(),
        parley::Alignment::Center,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
