use crate::cgi::Displayable;
use crate::cgi::coordinate::Coordinate;
use crate::cgi::widget::{Widget, WidgetHdl};
use std::collections::{HashMap, HashSet};

pub struct Layout {
    pub(crate) layout: HashMap<WidgetHdl, WidgetPlacement>,
}
pub(crate) struct RenderedLayout(pub HashMap<WidgetHdl, ComputedWidgetPlacement>);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WidgetPlacement {
    tl: (Coordinate, Coordinate),
    width: Coordinate,
    height: Coordinate,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ComputedWidgetPlacement {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
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
    pub fn new(x: Coordinate, y: Coordinate, width: Coordinate, height: Coordinate) -> Self {
        Self {
            tl: (x, y),
            width,
            height,
        }
    }

    pub fn fullscreen() -> Self {
        Self {
            tl: (Coordinate::Absolute(0), Coordinate::Absolute(0)),
            width: Coordinate::Relative(1.0),
            height: Coordinate::Relative(1.0),
        }
    }

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

    pub fn split(&self, amt_x: u32, amt_y: u32, out: &mut [Self]) {
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
    }

    pub fn expand_or_shrink<C: Into<Coordinate>>(&self, x: C, y: C) -> Self {
        let x = x.into();
        let y = y.into();
        
        self.shift_top_left(-x, -y).shift_bottom_right(x, y)
    }
}

impl Layout {
    pub fn new() -> Self {
        Self {
            layout: HashMap::new(),
        }
    }

    pub fn with_widget(mut self, widget: &Widget<impl Displayable + 'static>, placement: WidgetPlacement) -> Self {
        self.add_widget(widget, placement);
        self
    }    

    pub fn add_widget(&mut self, widget: &Widget<impl Displayable + 'static>, placement: WidgetPlacement) {
        let widget_hdl = WidgetHdl { widget: widget.as_dyn() };
        self.layout.insert(
            widget_hdl,
            placement
        );
    }

    pub(crate) fn render(self, size_x: i32, size_y: i32) -> RenderedLayout {
        let mut rendered_layout = HashMap::new();

        for (widget_hdl, layout_data) in self.layout {
            let x = layout_data.tl.0.compute_at(size_x);
            let y = layout_data.tl.1.compute_at(size_y);
            let width = layout_data.width.compute_at(size_x);
            let height = layout_data.height.compute_at(size_y);

            rendered_layout.insert(
                widget_hdl,
                ComputedWidgetPlacement {
                    x,
                    y,
                    width,
                    height,
                },
            );
        }

        RenderedLayout(rendered_layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy {
        data: u32,
    }

    impl Displayable for Dummy {
        fn display(&self) {
            println!("Displaying CustomWidget with data: {}", self.data);
        }

        fn name(&self) -> String {
            format!("CustomWidget {}", self.data)
        }
        
        fn get_changed_chars(&self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>)  {
            todo!()
        }
    }

    impl Dummy {
        fn new(data: u32) -> Self {
            Self { data }
        }
    }

    struct DummyGenerator {
        count: u32,
    }

    impl DummyGenerator {
        fn new() -> Self {
            Self { count: 0 }
        }

        fn next(&mut self) -> Dummy {
            let dummy = Dummy::new(self.count);
            self.count += 1;
            dummy
        }

        fn get_n_widgets(&mut self, n: u32) -> Vec<Widget<Dummy>> {
            (0..n).map(|_| Widget::new(self.next())).collect()
        }
    }

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
        use crate::cgi::layout;

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
            let placement = WidgetPlacement::new(Absolute(10), Absolute(20), Relative(1.0), Relative(0.8));
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
            let fs = WidgetPlacement::fullscreen().expand_or_shrink(5, 5);
            fs.split(3, 2, &mut placements);
            
            for i in 0..6 {
                layout.add_widget(&widgets[i], placements[i as usize]);
            }

            let rendered_layout = layout.render(310, 210);
            assert_eq!(fs.tl, (Absolute(5), Absolute(5)));
            assert_eq!(fs.width, Hybrid(-10, 1.0));
            assert_eq!(fs.height, Hybrid(-10, 1.0));
            for i in 0..6 {
                let widget_layout = unsafe { get_widget_from_rendered_layout(&rendered_layout, i) }.unwrap();
                assert_eq!(widget_layout.width, 100);
                assert_eq!(widget_layout.height, 100);
            }
        }
    }
}
