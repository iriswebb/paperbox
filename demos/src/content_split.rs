use alloc::{boxed::Box, sync::Arc, vec::Vec};
use embedded_graphics::{prelude::*, primitives::Rectangle};

#[derive(Clone)]
pub struct Widget<D: DrawTarget, C: Clone> {
    pub content: WidgetType<D, C>,
    pub padding: FourSizes,
}

pub trait Demo<D: DrawTarget, C: Clone> {
    fn render(&self, target: &mut D, r: Rectangle, context: C);
}

#[derive(Clone)]
pub enum WidgetType<D: DrawTarget, C: Clone> {
    Split(Split<D, C>),
    Tile(Arc<dyn Demo<D, C>>),
}

#[derive(Clone)]
pub struct Split<D: DrawTarget, C: Clone> {
    pub is_horiz: bool,
    pub widgets: Vec<Widget<D, C>>,
    pub ratios: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct FourSizes(pub u16, pub u16, pub u16, pub u16);

pub fn render<D: DrawTarget, C: Clone>(
    target: &mut D,
    widget: &Widget<D, C>,
    r: Rectangle,
    context: C,
) {
    let Rectangle {
        top_left: Point { x, y },
        size: Size {
            width: w,
            height: h,
        },
    } = r;
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
                    render(
                        target,
                        wid,
                        Rectangle::new(Point::new(offset as i32, y), Size::new(total as u32, h)),
                        context.clone(),
                    );
                    offset += total;
                } else {
                    let total = proportional * h as f32;
                    render(
                        target,
                        wid,
                        Rectangle::new(Point::new(x, offset as i32), Size::new(w, total as u32)),
                        context.clone(),
                    );
                    offset += total;
                }
            }
        }
        WidgetType::Tile(f) => f.render(target, r, context.clone()),
    }
}
