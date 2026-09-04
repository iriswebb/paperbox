#[macro_export]
macro_rules! split {
    ($t:expr, $i:expr) => {
        $crate::content_split::Widget {
            content: $crate::content_split::WidgetType::Split($crate::content_split::Split {
                is_horiz: $t,
                ratios: alloc::vec![1; $i.len()],
                widgets: alloc::vec::Vec::from($i),
            }),
            padding: $crate::content_split::FourSizes(0, 0, 0, 0),
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

#[macro_export]
macro_rules! from_demo {
    ($i:expr) => {
        $crate::content_split::Widget {
            content: $crate::content_split::WidgetType::Tile(alloc::sync::Arc::new($i)),
            padding: $crate::content_split::FourSizes(0, 0, 0, 0),
        }
    };
}
