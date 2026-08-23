// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A simple example that lays out some text using Parley, rasterises the glyph using Swash
//! and and then renders it into a PNG using the `image` crate.

#![expect(clippy::cast_possible_truncation, reason = "Deferred")]

use embedded_graphics::pixelcolor::{Gray4, Rgb888};
use embedded_graphics::prelude::*;
use parley::fontique::{Blob, FontInfoOverride};
use parley::layout::{Alignment, Glyph, GlyphRun, Layout, PositionedLayoutItem};
use parley::style::{FontFamily, StyleProperty};
use parley::{AlignmentOptions, FontContext, LayoutContext, LineHeight};
use std::alloc::GlobalAlloc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Scaler, Source, StrikeWith};
use swash::zeno;
use swash::FontRef;
use zeno::{Format, Vector};

/// Global atomic counter for tracking total allocated bytes
pub static BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

/// A custom global allocator that tracks memory allocations
pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        // Get memory from the system allocator
        let ret = std::alloc::System.alloc(layout);

        // Track the allocation if successful
        if !ret.is_null() {
            BYTES_ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }

        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        // Deallocate using the system allocator
        std::alloc::System.dealloc(ptr, layout);

        // Subtract from the counter
        BYTES_ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static TALLOC: TrackingAllocator = TrackingAllocator;

fn alpha_composite(background: Rgb888, foreground: Rgb888, alpha: u8) -> Rgb888 {
    let (r1, g1, b1) = (
        foreground.r() as u16,
        foreground.g() as u16,
        foreground.b() as u16,
    );
    let (r2, g2, b2) = (
        background.r() as u16,
        background.g() as u16,
        background.b() as u16,
    );

    let alpha = alpha as u16;
    let p = 255 - alpha;

    Rgb888::new(
        ((r1 * alpha + r2 * p) / 255) as u8,
        ((g1 * alpha + g2 * p) / 255) as u8,
        ((b1 * alpha + b2 * p) / 255) as u8,
    )
}

fn inverse(col: Rgb888) -> Rgb888 {
    let (r, g, b) = (col.r(), col.g(), col.b());
    Rgb888::new(Rgb888::MAX_R - r, Rgb888::MAX_G - g, Rgb888::MAX_B - b)
}

trait ParleyEngine:
    DrawTarget<Color: From<Rgb888> + Clone + PartialEq + Default + std::fmt::Debug + Into<Rgb888>>
{
}

impl<
        T: DrawTarget<
            Color: From<Rgb888> + Clone + PartialEq + Default + std::fmt::Debug + Into<Rgb888>,
        >,
    > ParleyEngine for T
{
}

pub fn draw_shaped_text<D: ParleyEngine>(
    display: &mut D,
    text: String,
    font_data: &'static [u8],
    font_size: f32,
    display_scale: f32,
    quantize: bool,
    width: Option<f32>,
    text_color: D::Color,
    aa_color: D::Color,
    padding: u32,
    line_height: f32,
    start: Point,
    alignment: Alignment,
) {
    // Create a FontContext, LayoutContext and ScaleContext
    // These are all intended to be constructed rarely (perhaps even once per app (or once per thread))
    // and provide caches and scratch space to avoid allocations
    let mut font_cx = FontContext::new();
    let mut layout_cx = LayoutContext::new();
    let mut scale_cx = ScaleContext::new();

    let blob = Blob::new(Arc::new(font_data));
    font_cx.collection.register_fonts(
        blob,
        Some(FontInfoOverride {
            family_name: Some("MyCustomFont"),
            ..FontInfoOverride::default()
        }),
    );

    let mut layout = {
        let mut builder = layout_cx.ranged_builder(&mut font_cx, &text, display_scale, quantize);
        builder.push_default(StyleProperty::Brush(text_color));
        builder.push_default(FontFamily::from("MyCustomFont"));
        builder.push_default(LineHeight::FontSizeRelative(line_height));
        builder.push_default(StyleProperty::FontSize(font_size));
        let layout: Layout<D::Color> = builder.build(&text);
        layout
    };

    // Perform layout (including bidi resolution and shaping) with start alignment
    layout.break_all_lines(width);
    layout.align(alignment, AlignmentOptions::default());

    // Iterate over laid out lines
    for line in layout.lines() {
        // Iterate over GlyphRun's within each line
        for glyph_run in line.items().map(|x| match x {
            PositionedLayoutItem::GlyphRun(g) => g,
            _ => panic!(),
        }) {
            render_glyph_run(&mut scale_cx, &glyph_run, display, padding);
        }
    }
}

fn render_glyph_run<D: ParleyEngine>(
    context: &mut ScaleContext,
    glyph_run: &GlyphRun<'_, D::Color>,
    img: &mut D,
    padding: u32,
) {
    // Resolve properties of the GlyphRun
    let mut run_x = glyph_run.offset();
    let run_y = glyph_run.baseline();
    let style = glyph_run.style();
    let color = style.brush;

    // Get the "Run" from the "GlyphRun"
    let run = glyph_run.run();

    // Resolve properties of the Run
    let font = run.font();
    let normalized_coords = run.normalized_coords();

    // Convert from parley::Font to swash::FontRef
    let font_ref = FontRef::from_index(font.data.as_ref(), font.index as usize).unwrap();

    // Build a scaler. As the font properties are constant across an entire run of glyphs
    // we can build one scaler for the run and reuse it for each glyph.
    let mut scaler = context
        .builder(font_ref)
        .size(run.font_size())
        .hint(true)
        .normalized_coords(normalized_coords)
        .build();

    // Iterates over the glyphs in the GlyphRun
    for glyph in glyph_run.glyphs() {
        let glyph_x = run_x + glyph.x + (padding as f32);
        let glyph_y = run_y + glyph.y + (padding as f32);
        run_x += glyph.advance;

        render_glyph(img, &mut scaler, color.into(), glyph, glyph_x, glyph_y);
    }
}

fn render_glyph<D: ParleyEngine>(
    img: &mut D,
    scaler: &mut Scaler<'_>,
    brush: Rgb888,
    glyph: Glyph,
    glyph_x: f32,
    glyph_y: f32,
) {
    // Compute the fractional offset
    // You'll likely want to quantize this in a real renderer
    let offset = Vector::new(glyph_x.fract(), glyph_y.fract());

    // Render the glyph using swash
    let rendered_glyph = Render::new(
        // Select our source order
        &[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ],
    )
    // Select the simple alpha (non-subpixel) format
    .format(Format::Alpha)
    // Apply the fractional offset
    .offset(offset)
    // Render the image
    .render(scaler, glyph.id as u16)
    .unwrap();

    let glyph_width = rendered_glyph.placement.width;
    let glyph_height = rendered_glyph.placement.height;
    let glyph_x = (glyph_x.floor() as i32 + rendered_glyph.placement.left) as u32;
    let glyph_y = (glyph_y.floor() as i32 - rendered_glyph.placement.top) as u32;

    match rendered_glyph.content {
        Content::Mask => {
            let mut i = 0;
            let bc = brush;
            for pixel_y in 0..glyph_height {
                for pixel_x in 0..glyph_width {
                    let x = glyph_x + pixel_x;
                    let y = glyph_y + pixel_y;
                    let alpha = rendered_glyph.data[i];
                    let c = alpha_composite(inverse(brush), brush, alpha);
                    Pixel(
                        Point {
                            x: x as i32,
                            y: y as i32,
                        },
                        c.into(),
                    )
                    .draw(img);
                    i += 1;
                }
            }
        }
        Content::SubpixelMask => unimplemented!(),
        Content::Color => {
            let row_size = glyph_width as usize * 4;
            for (pixel_y, row) in rendered_glyph.data.chunks_exact(row_size).enumerate() {
                for (pixel_x, pixel) in row.chunks_exact(4).enumerate() {
                    let x = glyph_x + pixel_x as u32;
                    let y = glyph_y + pixel_y as u32;
                    let color = alpha_composite(
                        inverse(brush),
                        Rgb888::new(pixel[0], pixel[1], pixel[2]),
                        pixel[3],
                    );
                    Pixel(
                        Point {
                            x: x as i32,
                            y: y as i32,
                        },
                        color.into(),
                    )
                    .draw(img);
                }
            }
        }
    }
}
