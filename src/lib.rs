#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(core_float_math)]

use core::ffi;

extern crate alloc;

use alloc::{ffi::CString, string::ToString};
use embedded_graphics::{
    geometry::Point,
    pixelcolor::{BinaryColor, Gray4, GrayColor},
};

use crate::content_split::{ContentDisplayDriver, EmbeddedGraphicsContentRenderer};

pub mod badapple;
pub mod cffi;
pub mod content_split;
pub mod content_split_macros;
pub mod content_split_vm;
pub mod shaped_text_demo;

extern "C" {
    pub fn drawPixel(x: ffi::c_int, y: ffi::c_int, color: ffi::c_uint);
    pub fn fillRect(x: i32, y: i32, w: i32, h: i32, color: u32);
    pub fn update();
    pub fn initGrayMode();
    pub fn deinitGrayMode();
    pub fn updataPartial(x: u16, y: u16, w: u16, h: u16);
    pub fn ffi_delay(ms: i32);
    pub fn drawBitmap(x: i16, y: i16, bitmap: *const u8, w: i16, h: i16, color: u16);
    pub fn println(x: &ffi::CStr);
    pub fn malloc(x: usize) -> *mut u8;
    pub fn free(x: *mut u8);
}

pub const WHITE: u32 = 0xFFFF;
pub const BLACK: u32 = 0;

pub const DISPLAY: cffi::Display = cffi::Display;
pub const GRAYDISPLAY: cffi::FourGrayDisplay = cffi::FourGrayDisplay;

unsafe fn fillScreen(c: u32) {
    fillRect(0, 0, 800, 480, c);
}

const ITERLIMIT: u32 = 1000;

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

/// # Safety
///
/// This function should be safe, and is unsafe only due to externed C functions
pub unsafe fn draw_badapple(fc: ffi::c_int) {
    use badapple::BADAPPLE;
    drawBitmap(
        0,
        0,
        BADAPPLE[fc as usize % const { BADAPPLE.len() }].as_ptr(),
        480,
        360,
        BLACK as u16,
    );
    updataPartial(0, 0, 480, 320);
}

const MANDLERANGE_X: (f32, f32) = (-2.5, 0.5);
const MANDLERANGE_Y: (f32, f32) = (-1.2, 1.2);

/// Transposes a range (x..y..step) onto a range (a..b), returning the step required for the second range
///
/// Ex:
///     (0..100..1) transposed onto (-1.0..1.0) has a step of 0.02, so the correct range is (-1.0..1.0..0.02)
///
/// Useful in scaling, transformation, and for floating point functions
pub const fn range_transpose(x: f32, y: f32, step: f32, a: f32, b: f32) -> f32 {
    use core::f32::math::ceil;
    (b - a) / ceil((y - x) / step)
}

/// # Safety
///
/// This function should be safe, and is unsafe only due to externed C functions
pub unsafe fn draw_mandlebrot() {
    let mandle_step_x = range_transpose(0f32, 800f32, 1f32, MANDLERANGE_X.0, MANDLERANGE_X.1);
    let mandle_step_y = range_transpose(0f32, 480f32, 1f32, MANDLERANGE_Y.0, MANDLERANGE_Y.1);
    for y in 0..480 {
        for x in 0..800 {
            let m = mandlebrot(
                MANDLERANGE_X.0 + (mandle_step_x * x as f32),
                MANDLERANGE_Y.0 + (mandle_step_y * y as f32),
            );
            drawPixel(x, y, if m { BLACK } else { WHITE });
        }
    }
    update();
    ffi_delay(1000);
}

static mut ZLIB_DECOMP_BUF: [u8; 48000] = [0; 48000];
pub const TST_IMG: &[u8] = include_bytes!("packed_image_data/o4x4_3.packed");
pub const PACKED_IMAGE_ZLIB: &[u8] = include_bytes!("packed_image_data/o4x4_3.packed.zlib");

/// :D
/// # Safety
///
/// This function should be safe, and is unsafe only due to externed C functions
pub unsafe fn zlib_test() {
    use zlib_rs::decompress_slice;
    let (_decomp, rc) = decompress_slice(
        &mut ZLIB_DECOMP_BUF,
        PACKED_IMAGE_ZLIB,
        zlib_rs::InflateConfig::default(),
    );
    assert_eq!(rc, zlib_rs::ReturnCode::Ok);
    drawBitmap(0, 0, ZLIB_DECOMP_BUF.as_ptr(), 800, 480, 0);
    update();
}

/// # Safety
///
/// This function should be safe, and is unsafe only due to externed C functions
unsafe fn embedded_graphics_widget_test() {
    let mut disp = cffi::Display;
    let mut eg = EmbeddedGraphicsContentRenderer(&mut disp);
    let widget = hsplit!([text!("meow"), vsplit!([text!("mrrp"), text!("mrrow"),])]);
    eg.render(&widget, 0, 0, 800, 480);
    update();
}

unsafe fn grey_demo() {
    initGrayMode();
    fillRect(100, 100, 200, 200, 0x0000);
    fillRect(0, 0, 100, 100, 0x2222);
    // fillRect(300, 100, 200, 200, 0x7BEF);
    // fillRect(300, 100, 200, 200, 0x0000);
    // fillRect(300, 100, 200, 200, 0x2222);
    update();
    deinitGrayMode();
}

const TTF_FONT_DATA: &[u8] = include_bytes!("fonts/LiberationSerif-Italic.ttf") as &[u8];

unsafe fn ttf_demo() {
    use embedded_graphics::{
        pixelcolor::{BinaryColor, Gray4, Rgb888},
        prelude::*,
        text::{Alignment, Text},
    };

    use alloc::sync::Arc;

    let f = Arc::new(ttf_parser::Face::parse(TTF_FONT_DATA, 0).unwrap());
    let font = femtofont::Font::from_face(f, femtofont::FontSettings::default()).unwrap();
    // Specify color and location
    let style =
        eg_femtofont::FemtoFontTextStyle::with_aa_color(&font, Gray4::BLACK, Gray4::WHITE, 140);
    let rendered_text =
        Text::with_alignment("FᴇᴍᴛᴏFᴏɴᴛ", Point::new(400, 240), style, Alignment::Center);

    // Render
    rendered_text.draw(&mut GRAYDISPLAY).unwrap();
}

unsafe fn shaped_ttf_demo() {
    shaped_text_demo::draw_shaped_text(
        &mut DISPLAY,
        String::from("Shaped Embedded Graphics\n\nالرسومات المدمجة"),
        include_bytes!("fonts/Rubik-Bold.ttf"),
        64.0,
        1.0,
        true,
        Some(790.0),
        BinaryColor::On,
        BinaryColor::Off,
        30,
        1.3,
        Point::zero(),
        parley::Alignment::Center,
    );
    update();
}

/// # Safety
///
/// This function should be safe, and is unsafe only due to externed C functions
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_setup() {
    println!("meow");

    // initGrayMode();
    fillScreen(WHITE);
    // Circle::new(Point::new(22, 22), 20).into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 20)).draw(&mut DISPLAY);
    update();
}

/// # Safety
///
/// This function should be safe, and is unsafe only due to externed C functions
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_loop(fc: ffi::c_int) {
    // grey_demo();
    shaped_ttf_demo();
}
