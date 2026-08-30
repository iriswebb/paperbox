#[macro_export]
macro_rules! text {
    ($e:expr) => {
        $crate::content_split::Widget {
            content: $crate::content_split::WidgetType::Text(
                $e.to_string(),
                $crate::content_split::DEFCOLOR,
                $crate::content_split::Alignment::Center,
                $crate::content_split::Font::MonoFont(&embedded_graphics::mono_font::ascii::FONT_10X20),
            ),
            padding: $crate::content_split::FourSizes(0, 0, 0, 0),
            margin: $crate::content_split::FourSizes(0, 0, 0, 0),
        }
    };
}

#[macro_export]
macro_rules! split {
    ($t:expr, $i:expr) => {
        $crate::content_split::Widget {
            content: $crate::content_split::WidgetType::Split($crate::content_split::Split {
                is_horiz: $t,
                ratios: alloc::vec![1; $i.len()],
                widgets: $i.to_vec(),
                border: $crate::content_split::Border::zero(),
            }),
            padding: $crate::content_split::FourSizes(0, 0, 0, 0),
            margin: $crate::content_split::FourSizes(0, 0, 0, 0),
        }
    };

    ($t:expr, $i:expr, $r:expr) => {
        $crate::content_split::Widget {
            content: $crate::content_split::WidgetType::Split($crate::content_split::Split {
                is_horiz: $t,
                ratios: &$r,
                widgets: &$i,
                border: $crate::content_split::Border::zero(),
            }),
            padding: $crate::content_split::FourSizes(0, 0, 0, 0),
            margin: $crate::content_split::FourSizes(0, 0, 0, 0),
        }
    };

    ($t:expr, $i:expr, $r:expr, $_s:expr) => {
        $crate::content_split::Widget {
            content: $crate::content_split::WidgetType::Split($crate::content_split::Split {
                is_horiz: $t,
                ratios: &$r,
                widgets: &$i,
                border: $crate::content_split::Border::zero(),
            }),
            padding: $crate::content_split::FourSizes(0, 0, 0, 0),
            margin: $crate::content_split::FourSizes(0, 0, 0, 0),
        }
    };
}

#[macro_export]
macro_rules! hsplit {
    ($i:expr) => {
        split!(true, $i)
    };

    ($i:expr, $r:expr) => {
        split!(true, $i, $r)
    };

    ($i:expr, $r:expr, $_s:expr) => {
        split!(true, $i, $r, $_s)
    };
}

#[macro_export]
macro_rules! vsplit {
    ($i:expr) => {
        split!(false, $i)
    };

    ($i:expr, $r:expr) => {
        split!(false, $i, $r)
    };

    ($i:expr, $r:expr, $_s:expr) => {
        split!(false, $i, $r, $_s)
    };
}
