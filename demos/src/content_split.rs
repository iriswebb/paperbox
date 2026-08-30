use alloc::{string::String, vec::Vec};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

#[derive(Clone)]
pub struct Widget {
    pub content: WidgetType,
    pub padding: FourSizes,
    pub margin: FourSizes,
}

#[derive(Clone)]
pub enum WidgetType {
    Split(Split),
    Text(String, Color, Alignment, Font),
    Image,
}

#[derive(Clone)]
pub struct Split {
    pub is_horiz: bool,
    pub widgets: Vec<Widget>,
    pub ratios: Vec<u8>,
    pub border: Border,
}

#[derive(Clone, Default)]
pub enum Image {
    FunctionImage,
    FloatingFunctionImage,
    #[default]
    BlankImage,
    PackedSliceImage(&'static [u8]),
    ZlibSliceImage(&'static [u8]),
    TiledSliceImage(&'static [u8], u16, u16),
    Color(Color),
}

#[derive(Debug, Clone)]
pub enum Font {
    MonoFont(&'static embedded_graphics::mono_font::MonoFont<'static>),
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct FourSizes(pub u16, pub u16, pub u16, pub u16);
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Border {
    pub color: Color,
    pub width: FourSizes,
}

impl Border {
    /// The Default trait isn't const
    pub const fn zero() -> Self {
        Border {
            color: Rgb888::WHITE,
            width: FourSizes(0, 0, 0, 0),
        }
    }
}
pub type Color = embedded_graphics::pixelcolor::Rgb888;
pub type Alignment = embedded_graphics::text::Alignment;

pub const DEFCOLOR: Rgb888 = Rgb888::BLACK;

pub trait ContentDisplayDriver {
    type Error;

    fn render_text(
        &mut self,
        t: &String,
        c: Color,
        a: Alignment,
        f: &Font,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Result<(), Self::Error>;
    fn render_image(&mut self, t: Image, x: u32, y: u32, w: u32, h: u32)
    -> Result<(), Self::Error>;

    fn render(
        &mut self,
        widget: &Widget,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Result<(), Self::Error> {
        match &widget.content {
            WidgetType::Split(s) => {
                let mut sum = 0f32;
                for i in &s.ratios {
                    sum += *i as f32;
                }
                let mut offset = if s.is_horiz { x } else { y } as f32;
                for (i, wid) in s.widgets.iter().enumerate() {
                    let proportional = (s.ratios.as_slice()[i] as f32) / sum;
                    if s.is_horiz {
                        let total = proportional * w as f32;
                        self.render(wid, offset as u32, y, total as u32, h)?;
                        offset += total;
                    } else {
                        let total = proportional * h as f32;
                        self.render(wid, x, offset as u32, w, total as u32)?;
                        offset += total;
                    }
                }
            }
            WidgetType::Text(t, c, a, f) => {
                self.render_text(t, *c, *a, f, x, y, w, h)?;
            }
            WidgetType::Image => self.render_image(Image::FunctionImage, x, y, w, h)?,
        }
        Ok(())
    }
}

pub struct EmbeddedGraphicsContentRenderer<'a, T: DrawTarget>(pub &'a mut T);

impl<T: DrawTarget<Color: From<Rgb888>>> ContentDisplayDriver
    for EmbeddedGraphicsContentRenderer<'_, T>
{
    type Error = T::Error;

    // Black Const Font Text Centered On a White Background
    fn render_text(
        &mut self,
        t: &String,
        c: Color,
        a: Alignment,
        f: &Font,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Result<(), Self::Error> {
        fn inverse(col: Rgb888) -> Rgb888 {
            let (r, g, b) = (col.r(), col.g(), col.b());
            Rgb888::new(Rgb888::MAX_R - r, Rgb888::MAX_G - g, Rgb888::MAX_B - b)
        }

        let bounding = Rectangle::new(
            Point {
                x: x as i32,
                y: y as i32,
            },
            Size::new(w, h),
        );

        let style = match f {
            Font::MonoFont(mono) => MonoTextStyle::new(&mono, c.into()),
        };

        let blankout = PrimitiveStyleBuilder::new()
            .stroke_width(0)
            .fill_color(inverse(c).into())
            .build();

        let text = Text::with_alignment(t, bounding.center(), style, a);

        bounding.into_styled(blankout).draw(self.0)?;
        text.draw(self.0)?;

        Ok(())
    }

    fn render_image(
        &mut self,
        _t: Image,
        _x: u32,
        _y: u32,
        _w: u32,
        _h: u32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
