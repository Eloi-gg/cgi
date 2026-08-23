use crate::Displayable;
use crate::coordinate::Coordinate;
use crate::widget::{Widget, WidgetHdl};
use std::collections::HashMap;

pub struct Layout {
    pub(crate) layout: HashMap<WidgetHdl, WidgetPlacement>,
}

pub(crate) struct RenderedLayout(pub(crate) HashMap<WidgetHdl, ComputedWidgetPlacement>);

impl RenderedLayout {
    pub fn get_widget_coords(&self, widget: &WidgetHdl, only_insides: bool) -> ComputedWidgetPlacement {
        let placement = self.0[widget];

        return if only_insides {
            let outlined = widget.widget.data.lock().unwrap().outline.is_some();
            if outlined {
                placement.shrinked()
            } else {
                placement
            }
        } else {
            placement
        };
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct WidgetPlacement {
    tl: (Coordinate, Coordinate),
    width: Coordinate,
    height: Coordinate,
}

pub struct WidgetGroupPlacement {
    //TODO REMOVE
    placement: WidgetPlacement,
    split_x: u32,
    split_y: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct ComputedWidgetPlacement {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ComputedWidgetPlacement {
    pub fn shrinked(self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y + 1,
            width: self.width - 2,
            height: self.height - 2,
        }
    }
}

impl Default for WidgetPlacement {
    fn default() -> Self {
        Self {
            tl: (Coordinate::Absolute(0), Coordinate::Absolute(0)),
            width: Coordinate::Absolute(0),
            height: Coordinate::Absolute(0),
        }
    }
}

impl WidgetPlacement {
    pub fn new<C: Into<Coordinate>>(x: C, y: C, width: C, height: C) -> Self {
        Self {
            tl: (x.into(), y.into()),
            width: width.into(),
            height: height.into(),
        }
    }

    pub fn fullscreen() -> Self {
        Self {
            tl: (Coordinate::Absolute(0), Coordinate::Absolute(0)),
            width: Coordinate::Relative(1.0),
            height: Coordinate::Relative(1.0),
        }
    }

    //TODO: the following functions are half builder pattern half mutators which is confusing

    pub fn shift_top_left<C: Into<Coordinate>>(&self, x: C, y: C) -> Self {
        let x = x.into();
        let y = y.into();
        Self {
            tl: (self.tl.0 + x, self.tl.1 + y),
            width: self.width - x,
            height: self.height - y,
        }
    }

    pub fn shift_bottom_right<C: Into<Coordinate>>(&self, x: C, y: C) -> Self {
        Self {
            tl: self.tl,
            width: self.width + x.into(),
            height: self.height + y.into(),
        }
    }

    pub fn shift<C: Into<Coordinate>>(&self, x: C, y: C) -> Self {
        Self {
            tl: (self.tl.0 + x.into(), self.tl.1 + y.into()),
            width: self.width,
            height: self.height,
        }
    }

    //TODO REMOVE
    pub fn split_into_group(&self, amt_x: u32, amt_y: u32) -> WidgetGroupPlacement {
        WidgetGroupPlacement {
            placement: self.clone(),
            split_x: amt_x,
            split_y: amt_y,
        }
    }

    /// Splits into several placements. out will be a list of columns
    pub fn split(&self, amt_x: u32, amt_y: u32, expand_edges: bool, out: &mut [Self]) {
        let width_per_split = self.width / amt_x as f32;
        let height_per_split = self.height / amt_y as f32;

        let unit = Self::new(self.tl.0, self.tl.1, width_per_split, height_per_split);

        for i in 0..amt_x {
            for j in 0..amt_y {
                let x = width_per_split * i as f32;
                let y = height_per_split * j as f32;
                out[(i * amt_y + j) as usize] = unit.shift(x, y);
            }
        }

        for i in 0..amt_x {
            out[(i * amt_y + (amt_y - 1)) as usize] =
                out[(i * amt_y + (amt_y - 1)) as usize].shift_bottom_right(0, 1);
        }
        for i in 0..amt_y {
            out[((amt_x - 1) * amt_y + i) as usize] =
                out[((amt_x - 1) * amt_y + i) as usize].shift_bottom_right(1, 0);
        }
    }

    /// Positive values expand the widget, negative values shrink it.
    pub fn expand_or_shrink<C: Into<Coordinate>>(&self, x: C, y: C) -> Self {
        let x = x.into();
        let y = y.into();

        self.shift_top_left(-x, -y).shift_bottom_right(x, y)
    }

    pub fn get_top_left(&self) -> (Coordinate, Coordinate) {
        self.tl
    }

    pub fn get_bottom_right(&self) -> (Coordinate, Coordinate) {
        (self.tl.0 + self.width, self.tl.1 + self.height)
    }

    pub fn with_x<C: Into<Coordinate>>(mut self, x: C) -> Self {
        self.tl.0 = x.into();
        self
    }

    pub fn with_y<C: Into<Coordinate>>(mut self, y: C) -> Self {
        self.tl.1 = y.into();
        self
    }

    pub fn with_width<C: Into<Coordinate>>(mut self, width: C) -> Self {
        self.width = width.into();
        self
    }

    pub fn with_height<C: Into<Coordinate>>(mut self, height: C) -> Self {
        self.height = height.into();
        self
    }

    pub fn get_below(&self) -> Self {
        Self {
            tl: (self.tl.0, self.tl.1 + self.height),
            width: self.width,
            height: Coordinate::Relative(1.0) - self.height - self.tl.1,
        }
    }

    pub fn get_right(&self) -> Self {
        Self {
            tl: (self.tl.0 + self.width, self.tl.1),
            width: Coordinate::Relative(1.0) - self.width - self.tl.0,
            height: self.height,
        }
    }

    pub fn get_left(&self) -> Self {
        Self {
            tl: (0.into(), self.tl.1),
            width: self.tl.0,
            height: self.height,
        }
    }

    pub fn get_above(&self) -> Self {
        Self {
            tl: (self.tl.0, 0.into()),
            width: self.width,
            height: self.tl.1,
        }
    }
}

impl Layout {
    pub fn new() -> Self {
        Self {
            layout: HashMap::new(),
        }
    }

    pub fn with_widget(
        mut self,
        widget: &Widget<impl Displayable + 'static>,
        placement: WidgetPlacement,
    ) -> Self {
        self.add_widget(widget, placement);
        self
    }

    pub fn add_widget(
        &mut self,
        widget: &Widget<impl Displayable + 'static>,
        placement: WidgetPlacement,
    ) {
        let widget_hdl = WidgetHdl {
            widget: widget.as_dyn(),
        };
        self.layout.insert(widget_hdl, placement);
    }

    pub fn connect_and_add_widgets(
        &mut self,
        widgets: &mut Vec<Widget<impl Displayable + 'static>>, //TODO: intoIterator
        placements: &mut [WidgetPlacement],
    ) {
        let mut locks = widgets
            .iter_mut()
            .map(|w| w.data.lock().unwrap())
            .collect::<Vec<_>>();
        locks.iter_mut().for_each(|x| x.connected = 0);
        // for each corner, stores the widgets that are connected to it and which direction they are connected in
        let mut connection_points: HashMap<(Coordinate, Coordinate), Vec<(usize, u8)>> =
            HashMap::new();

        const RIGHT: u8 = 1 << 0;
        const BOTTOM: u8 = 1 << 1;
        const LEFT: u8 = 1 << 2;
        const TOP: u8 = 1 << 3;

        for (idx, placement) in placements.into_iter().enumerate() {
            let tl = placement.get_top_left();
            let br = placement.get_bottom_right();
            let tr = (br.0, tl.1);
            let bl = (tl.0, br.1);

            match connection_points.get_mut(&tl) {
                Some(connected) => connected.push((idx, TOP | LEFT)),
                None => {
                    connection_points.insert(tl, vec![(idx, TOP | LEFT)]);
                }
            }
            match connection_points.get_mut(&tr) {
                Some(connected) => connected.push((idx, TOP | RIGHT)),
                None => {
                    connection_points.insert(tr, vec![(idx, TOP | RIGHT)]);
                }
            }
            match connection_points.get_mut(&bl) {
                Some(connected) => connected.push((idx, BOTTOM | LEFT)),
                None => {
                    connection_points.insert(bl, vec![(idx, BOTTOM | LEFT)]);
                }
            }
            match connection_points.get_mut(&br) {
                Some(connected) => connected.push((idx, BOTTOM | RIGHT)),
                None => {
                    connection_points.insert(br, vec![(idx, BOTTOM | RIGHT)]);
                }
            }
        }

        let mut expand = vec![(false, false); locks.len()];

        for connected in connection_points.values_mut() {
            if connected.len() <= 1 {
                continue;
            }

            let connection_type = connected
                .iter()
                .fold(u8::MAX, |acc, (_, flags)| acc & flags);
            let mut is_vertical = ((connection_type & (LEFT | RIGHT)) > 0) as u8;
            let mut is_lateral = ((connection_type & (TOP | BOTTOM)) > 0) as u8;
            if is_lateral == 0 && is_vertical == 0 {
                is_vertical = 1;
                is_lateral = 1;
            }
            let connection_type = is_lateral * crate::widget::connections::CONNECTED_LATERAL
                | is_vertical * crate::widget::connections::CONNECTED_VERTICAL;
            for (idx, local_connection) in connected.iter() {
                let offset = (*local_connection & (RIGHT | BOTTOM)) * 2;
                locks[*idx].connected |= connection_type << offset;

                // expand the widget by 1
                let expand_bottom = local_connection & BOTTOM * is_vertical > 0;
                let expand_right = local_connection & RIGHT * is_lateral > 0;

                expand[*idx].0 |= expand_bottom;
                expand[*idx].1 |= expand_right;
            }
        }
        drop(locks);

        for (i, p) in placements.into_iter().enumerate() {
            let (b, r) = expand[i];
            let b = b as i32;
            let r = r as i32;

            *p = p.shift_bottom_right(r, b);
        }

        for (widget, placement) in widgets.iter().zip(placements.into_iter()) {
            self.add_widget(widget, *placement);
        }
    }

    pub(crate) fn render(&self, size_x: i32, size_y: i32) -> RenderedLayout {
        let mut rendered_layout = HashMap::new();

        for (widget_hdl, layout_data) in self.layout.iter() {
            let x = layout_data.tl.0.compute_at(size_x);
            let y = layout_data.tl.1.compute_at(size_y);
            let width = layout_data.width.compute_at(size_x);
            let height = layout_data.height.compute_at(size_y);

            rendered_layout.insert(
                widget_hdl.clone(),
                ComputedWidgetPlacement {
                    x,
                    y,
                    width,
                    height,
                },
            );
        }

        RenderedLayout::new(rendered_layout)
    }
}

impl std::fmt::Debug for ComputedWidgetPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{}) {}*{}", self.x, self.y, self.width, self.height)
    }
}

impl std::fmt::Debug for WidgetPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({:?},{:?}) {:?}*{:?}",
            self.tl.0, self.tl.1, self.width, self.height
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::test::{Dummy, DummyGenerator};

    #[test]
    fn coordinate_addition() {
        use Coordinate::*;

        assert_eq!(Absolute(5) + Absolute(10), Absolute(15));
        assert_eq!(Relative(0.2) + Relative(0.3), Relative(0.5));
        assert_eq!(Hybrid(5, 0.2) + Hybrid(10, 0.3), Hybrid(15, 0.5));
        assert_eq!(Absolute(1) + Relative(0.2), Hybrid(1, 0.2));
        assert_eq!(Relative(0.2) + Absolute(1), Hybrid(1, 0.2));
        assert_eq!(Hybrid(1, 0.2) + Absolute(5), Hybrid(6, 0.2));
        assert_eq!(Hybrid(1, 0.2) + Relative(0.3), Hybrid(1, 0.5));
    }

    mod layout {
        use crate::layout;

        use super::super::*;
        use super::Coordinate::*;

        // Looks through the computed_layout to find a Dummy widget with its data matching widget_data
        // ComputedLayout MUST contain only Dummy
        unsafe fn get_widget(layout: &Layout, widget_data: u32) -> Option<&WidgetPlacement> {
            for (widget_hdl, layout_data) in &layout.layout {
                let r = widget_hdl.widget.displayable.read().unwrap();
                let x = &*r as *const dyn Displayable as *const tests::Dummy;
                let current_widget_data = unsafe { (*x).data };

                if current_widget_data == widget_data {
                    return Some(layout_data);
                }
            }
            None
        }

        unsafe fn get_widget_from_rendered_layout(
            layout: &RenderedLayout,
            widget_data: u32,
        ) -> Option<&ComputedWidgetPlacement> {
            for (widget_hdl, placement) in &layout.0 {
                let r = widget_hdl.widget.displayable.read().unwrap();
                let x = &*r as *const dyn Displayable as *const tests::Dummy;
                let current_widget_data = unsafe { (*x).data };

                if current_widget_data == widget_data {
                    return Some(&placement);
                }
            }
            None
        }

        #[test]
        fn simplest_layout() {
            let mut layout = Layout::new();
            let mut dg = super::DummyGenerator::new();
            let widgets = dg.get_n_widgets(1);
            let placement =
                WidgetPlacement::new(Absolute(10), Absolute(20), Relative(1.0), Relative(0.8));
            layout.add_widget(&widgets[0], placement);

            let w1_layout_data = unsafe { get_widget(&layout, 0) };

            assert_eq!(w1_layout_data.unwrap().tl.0, Absolute(10));
            assert_eq!(w1_layout_data.unwrap().tl.1, Absolute(20));
            assert_eq!(w1_layout_data.unwrap().width, Relative(1.0));
            assert_eq!(w1_layout_data.unwrap().height, Relative(0.8));

            let rendered_layout = layout.render(100, 100);

            let widget = rendered_layout.0.iter().next().unwrap();
            assert_eq!(widget.1.x, 10);
            assert_eq!(widget.1.y, 20);
            assert_eq!(widget.1.width, 100);
            assert_eq!(widget.1.height, 100 * 8 / 10);
        }

        #[test]
        fn split_screen() {
            let mut layout = Layout::new();
            let mut dg = super::DummyGenerator::new();
            let widgets = dg.get_n_widgets(6);

            let mut placements = [WidgetPlacement::default(); 6];
            let fs = WidgetPlacement::fullscreen().expand_or_shrink(-5, -5);
            fs.split(3, 2, false, &mut placements);

            for i in 0..6 {
                layout.add_widget(&widgets[i], placements[i as usize]);
            }

            let rendered_layout = layout.render(310, 210);
            assert_eq!(fs.tl, (Absolute(5), Absolute(5)));
            assert_eq!(fs.width, Hybrid(-10, 1.0));
            assert_eq!(fs.height, Hybrid(-10, 1.0));
            for i in 0..6 {
                let widget_layout =
                    unsafe { get_widget_from_rendered_layout(&rendered_layout, i) }.unwrap();
                assert_eq!(widget_layout.x, 5 + ((i / 2) as i32 * 100));
                assert_eq!(widget_layout.y, 5 + ((i % 2) as i32 * 100));
                assert_eq!(widget_layout.width, 100);
                assert_eq!(widget_layout.height, 100);
            }
        }
    }
}
