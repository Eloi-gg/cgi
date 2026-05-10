use crate::cgi::Displayable;
use crate::cgi::coordinate::Coordinate;
use crate::cgi::widget::{Widget, WidgetHdl};
use std::collections::{HashMap, HashSet};

pub struct Layout {
    pub(crate) layout: HashMap<WidgetHdl, WidgetPlacement>,
}
pub(crate) struct RenderedLayout(HashMap<WidgetHdl, ComputedWidgetPlacement>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetPlacement {
    tl: (Coordinate, Coordinate),
    width: Coordinate,
    height: Coordinate,
}

#[derive(Debug, Clone, Copy)]
struct ComputedWidgetPlacement {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}


impl WidgetPlacement {
    fn new(x: Coordinate, y: Coordinate, width: Coordinate, height: Coordinate) -> Self {
        Self {
            tl: (x, y),
            width,
            height,
        }
    }
}

impl Layout {
    pub fn new() -> Self {
        Self {
            layout: HashMap::new(),
        }
    }

    pub fn add_widget(&mut self, widget: &Widget<impl Displayable + 'static>, placement: WidgetPlacement) {
        let widget_hdl = WidgetHdl { widget: widget.as_dyn() };
        self.layout.insert(
            widget_hdl,
            placement
        );
    }

    fn render(self, size_x: i32, size_y: i32) -> RenderedLayout {
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
        assert_eq!(Adaptative(0) + Absolute(5), Adaptative(5));
        assert_eq!(Relative(0.2) + Adaptative(0), Adaptative(0));
    }

    mod layout {
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
            assert_eq!(widget.1.width, (100 - 10));
            assert_eq!(widget.1.height, (100 - 20) * 8 / 10);
        }
    }
}
