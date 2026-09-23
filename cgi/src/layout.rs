use crate::Displayable;
use crate::coordinate::Coordinate;
use crate::widget::{Widget, WidgetHdl};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub(crate) struct Mask {
    data: Vec<u32>,
    width: usize,
}

impl Mask {
    fn new_transparent(width: usize, height: usize) -> Self {
        let data = vec![u32::MAX; (1 + (width / 32)) * height];
        Mask { data, width }
    }

    fn add(&mut self, coordinate: ComputedWidgetPlacement) {
        // Line mask is a reversed bitmask for the line, so that we can AND its complement with the data
        let mut line_mask = vec![u32::MAX >> coordinate.x];
        while line_mask.len() <= (coordinate.width / 32) as usize {
            line_mask.push(u32::MAX);
        }
        *line_mask.last_mut().unwrap() ^= u32::MAX >> ((coordinate.x + coordinate.width) % 32);
        while line_mask.len() < (self.width / 32) + 1 {
            line_mask.push(0);
        }

        let data_width = line_mask.len();

        for line_y in coordinate.y..(coordinate.y + coordinate.height) {
            let line_offset = line_y as usize * data_width;
            for (data_x, mask) in line_mask.iter().enumerate() {
                let data_offset = line_offset + data_x;
                self.data[data_offset] &= !*mask;
            }
        }
    }

    fn combined(&self, other: &Mask) -> Mask {
        assert_eq!(self.data.len(), other.data.len());

        let mut result = self.clone();
        result
            .data
            .iter_mut()
            .zip(other.data.iter())
            .for_each(|(a, b)| *a |= *b);
        result
    }

    pub fn at(&self, x: usize, y: usize) -> bool {
        let line_offset = y as usize * (1 + (self.width / 32)) + x / 32;
        let data = self.data[line_offset];
        let bit_offset = x % 32;
        (data & (1 << (31 - bit_offset))) != 0
    }
}

struct MaskStack {
    unit_masks: Vec<Mask>,
    combined_masks: Vec<Mask>,
}

impl MaskStack {
    pub fn new(mask: Mask) -> Self {
        MaskStack {
            unit_masks: vec![mask.clone()],
            combined_masks: vec![mask],
        }
    }

    pub fn push(&mut self, mask: Mask) {
        self.unit_masks.push(mask.clone());
        self.combined_masks
            .push(self.combined_masks.last().unwrap().combined(&mask));
    }

    pub fn pop(&mut self) {
        self.unit_masks.pop();
        self.combined_masks.pop();
    }

    pub fn set_mask(&mut self, index: usize, mask: Mask) {
        self.unit_masks[index] = mask.clone();
        for i in index..self.combined_masks.len() {
            self.combined_masks[i] = self.combined_masks[i - 1].combined(&self.combined_masks[i]);
        }
    }
}

impl std::ops::Index<usize> for MaskStack {
    type Output = Mask;

    fn index(&self, index: usize) -> &Self::Output {
        &self.combined_masks[index]
    }
}

pub struct Layout {
    pub(crate) layers: Vec<HashMap<WidgetHdl, WidgetPlacement>>,
    pub(crate) current_layer: usize, // pub(crate) vertical_connections: HashSet<(WidgetHdl, WidgetHdl)>,
}

pub(crate) struct RenderedLayout {
    pub(crate) layers: Vec<HashMap<WidgetHdl, ComputedWidgetPlacement>>,
    pub(crate) layer_mapping: HashMap<WidgetHdl, usize>,
    pub(crate) current_layer: usize, //TODO: is this even used
}

impl RenderedLayout {
    pub fn new_empty() -> Self {
        RenderedLayout {
            layers: vec![HashMap::new()],
            layer_mapping: HashMap::new(),
            current_layer: 0,
        }
    }

    pub fn get_widget_coords(
        &self,
        widget: &WidgetHdl,
        only_insides: bool,
    ) -> ComputedWidgetPlacement {
        let placement = self.layers[self.current_layer][widget];

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

    pub fn full_iter(&self) -> impl Iterator<Item = (&WidgetHdl, &ComputedWidgetPlacement)> {
        self.layers.iter().flat_map(|l| l.iter())
    }

    pub fn full_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&WidgetHdl, &mut ComputedWidgetPlacement)> {
        self.layers.iter_mut().flat_map(|l| l.iter_mut())
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct WidgetPlacement {
    tl: (Coordinate, Coordinate),
    width: Coordinate,
    height: Coordinate,
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

    pub fn is_null(&self) -> bool {
        self.width <= 0 || self.height <= 0
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
    pub fn new_with_size<C: Into<Coordinate>>(x: C, y: C, width: C, height: C) -> Self {
        Self {
            tl: (x.into(), y.into()),
            width: width.into(),
            height: height.into(),
        }
    }

    pub fn new_with_points<C: Into<Coordinate> + Copy>(
        x: C,
        y: C,
        bottom_right_x: C,
        bottom_right_y: C,
    ) -> Self {
        let width = bottom_right_x.into() - x.into();
        let height = bottom_right_y.into() - y.into();

        Self {
            tl: (x.into(), y.into()),
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

    /// Splits into several placements. out will be a list of columns
    pub fn split(&self, amt_x: u32, amt_y: u32, expand_edges: bool, out: &mut [Self]) {
        let width_per_split = self.width / amt_x as f32;
        let height_per_split = self.height / amt_y as f32;

        let unit = Self::new_with_size(self.tl.0, self.tl.1, width_per_split, height_per_split);

        for i in 0..amt_x {
            for j in 0..amt_y {
                let x = width_per_split * i as f32;
                let y = height_per_split * j as f32;
                out[(i * amt_y + j) as usize] = unit.shift(x, y);
            }
        }

        if expand_edges {
            for i in 0..amt_x {
                out[(i * amt_y + (amt_y - 1)) as usize] =
                    out[(i * amt_y + (amt_y - 1)) as usize].shift_bottom_right(0, 1);
            }
            for i in 0..amt_y {
                out[((amt_x - 1) * amt_y + i) as usize] =
                    out[((amt_x - 1) * amt_y + i) as usize].shift_bottom_right(1, 0);
            }
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

    pub fn with_bottom_right_x<C: Into<Coordinate>>(mut self, x: C) -> Self {
        self.width = x.into() - self.tl.0;
        self
    }

    pub fn with_bottom_right_y<C: Into<Coordinate>>(mut self, y: C) -> Self {
        self.height = y.into() - self.tl.1;
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
            layers: vec![HashMap::new()],
            current_layer: 0, // vertical_connections: HashSet::new(),
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

    pub fn set_layer(&mut self, layer_id: usize) {
        while self.layers.len() <= layer_id {
            self.layers.push(HashMap::new());
        }

        self.current_layer = layer_id;
    }

    pub fn add_widget(
        &mut self,
        widget: &Widget<impl Displayable + 'static>,
        placement: WidgetPlacement,
    ) {
        let widget_hdl = widget.as_hdl();
        self.layers[self.current_layer].insert(widget_hdl, placement);
    }

    pub fn connect_and_add_widgets(
        &mut self,
        widgets: &mut [Widget<impl Displayable + 'static>], //TODO: intoIterator
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
        let mut layers = Vec::new();
        let mut layer_mapping = HashMap::new();

        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let mut rendered_layer = HashMap::new();
            for (widget_hdl, layout_data) in layer.iter() {
                let x = layout_data.tl.0.compute_at(size_x);
                let y = layout_data.tl.1.compute_at(size_y);
                let max_width = size_x - x;
                let max_height = size_y - y;
                let width = layout_data.width.compute_at(size_x).min(max_width);
                let height = layout_data.height.compute_at(size_y).min(max_height);
                if width > 0 && height > 0 {
                    layer_mapping.insert(widget_hdl.clone(), layer_idx);
                    rendered_layer.insert(
                        widget_hdl.clone(),
                        ComputedWidgetPlacement {
                            x,
                            y,
                            width,
                            height,
                        },
                    );
                }
            }
            layers.push(rendered_layer);
        }

        let mut r = RenderedLayout {
            layers,
            layer_mapping,
            current_layer: self.current_layer,
        };

        // All this is just to send a resize event to the widgets
        let mut actions = crate::ActionList::new();
        for (widget, placement) in r.full_iter_mut() {
            let inside_placement = if let Ok(data) = widget.widget.data.lock() {
                if (*data).outline.is_some() {
                    ComputedWidgetPlacement {
                        x: placement.x + 1,
                        y: placement.y + 1,
                        width: placement.width - 2,
                        height: placement.height - 2,
                    }
                } else {
                    *placement
                }
            } else {
                *placement
            };
            widget.widget.displayable.write().unwrap().on_event(
                crate::Event::Resize(
                    inside_placement.width as u16,
                    inside_placement.height as u16,
                ),
                &mut actions,
            );
        }

        r
    }

    pub fn append(&mut self, other: Layout) {
        for (layer_idx, other_layer) in other.layers.iter().enumerate() {
            if self.layers.len() <= layer_idx {
                self.layers.push(HashMap::new());
            }
            for (widget_hdl, layout_data) in other_layer {
                if !self.layers[layer_idx].contains_key(&widget_hdl) {
                    self.layers[layer_idx].insert(widget_hdl.clone(), *layout_data);
                }
            }
        }
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
        use super::super::*;
        use super::Coordinate::*;
        use crate::factory_widgets::Listener;
        use crate::rendering::TestOutput;
        use crate::symbols::OutlineStyle;
        use crate::test;
        use crate::test::{FillGenerator, assert_match_with_test_file};
        use crate::{ActionList, WidgetBuilder, factory_widgets, layout};

        // Looks through the computed_layout to find a Dummy widget with its data matching widget_data
        // ComputedLayout MUST contain only Dummy
        unsafe fn get_widget(layout: &Layout, widget_data: u32) -> Option<&WidgetPlacement> {
            for (widget_hdl, layout_data) in &layout.layers[layout.current_layer] {
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
            for (widget_hdl, placement) in &layout.layers[layout.current_layer] {
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
            let placement = WidgetPlacement::new_with_size(
                Absolute(10),
                Absolute(20),
                Relative(1.0),
                Relative(0.8),
            );
            layout.add_widget(&widgets[0], placement);

            let w1_layout_data = unsafe { get_widget(&layout, 0) };

            assert_eq!(w1_layout_data.unwrap().tl.0, Absolute(10));
            assert_eq!(w1_layout_data.unwrap().tl.1, Absolute(20));
            assert_eq!(w1_layout_data.unwrap().width, Relative(1.0));
            assert_eq!(w1_layout_data.unwrap().height, Relative(0.8));

            let rendered_layout = layout.render(100, 100);

            let widget = rendered_layout.layers[0].iter().next().unwrap();
            assert_eq!(widget.1.x, 10);
            assert_eq!(widget.1.y, 20);
            assert_eq!(widget.1.width, 90); // Width 100 but gets clipped
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

        #[test]
        fn layering() {
            let mut output = TestOutput::<16, 8>::new();
            let mut layout = Layout::new();

            // WIDGETS
            let text_box_widget = WidgetBuilder::new(factory_widgets::text::TextBox::new(
                &test::strings::lorem_ipsum_long(),
                Listener::empty(),
                factory_widgets::text::TextAlign::Left,
            ))
            .with_outline(OutlineStyle::Thick)
            .build();
            let (fill_widget_back, fill_widget_front) = {
                let mut v = FillGenerator::new().get_n_widgets(2);
                let mut f = v.pop().unwrap();
                let b = v.pop().unwrap();
                f.set_outline(OutlineStyle::Double);
                (b, f)
            };
            let transparent_widget = WidgetBuilder::new(factory_widgets::text::TextBox::new(
                "@   ",
                Listener::empty(),
                factory_widgets::text::TextAlign::Left,
            ))
            .with_outline(OutlineStyle::Rounded)
            .transparent()
            .build();
            let not_transparent_widget = WidgetBuilder::new(factory_widgets::text::TextBox::new(
                "@",
                Listener::empty(),
                factory_widgets::text::TextAlign::Left,
            ))
            .with_outline(OutlineStyle::Rounded)
            .build();

            // PLACEMENTS
            let text_box_placement = WidgetPlacement::fullscreen();
            let fill_back_placement = WidgetPlacement::fullscreen().expand_or_shrink(-3, -2);
            let fill_front_placement = WidgetPlacement::new_with_size(3, 1, 6, 3);
            let transparent_placement = WidgetPlacement::new_with_size(3, 4, 6, 3);
            let not_transparent_placement = transparent_placement.shift(6, 0);

            layout.set_layer(0);
            layout.add_widget(&fill_widget_back, fill_back_placement);
            layout.set_layer(1);
            layout.add_widget(&text_box_widget, text_box_placement);
            layout.set_layer(2);
            layout.add_widget(&fill_widget_front, fill_front_placement);
            layout.add_widget(&transparent_widget, transparent_placement);
            layout.add_widget(&not_transparent_widget, not_transparent_placement);

            // RENDERING
            let layout = layout.render(16, 8);
            output.clear();
            layout.full_render_to_output(&mut output);
            let rendered_text = output.to_string();
            println!("{}", rendered_text);
            assert_match_with_test_file(&rendered_text, "12_layering");
        }

        #[test]
        fn mask() {
            // REVERSED:
            // 1100     0000     1100
            // 1100  +  0111  =  1111
            // 0000     0111     0111
            // 0000     0000     0000
            // EXPECTED:
            // 0011
            // 0000
            // 1000
            // 1111

            let c1 = ComputedWidgetPlacement {
                x: 0,
                y: 0,
                width: 2,
                height: 2,
            };
            let c2 = ComputedWidgetPlacement {
                x: 1,
                y: 1,
                width: 3,
                height: 2,
            };

            let mut mask = Mask::new_transparent(4, 4);
            mask.add(c1);
            mask.add(c2);
            mask.data.iter().for_each(|e| println!("{:032b}", e));
            assert_eq!(
                mask.data,
                vec![
                    !(0b1100 << 28),
                    !(0b1111 << 28),
                    !(0b0111 << 28),
                    !(0b0000 << 28)
                ]
            );

            let expected = [[0, 0, 1, 1], [0, 0, 0, 0], [1, 0, 0, 0], [1, 1, 1, 1]];

            for (line_idx, line) in expected.into_iter().enumerate() {
                line.iter().enumerate().for_each(|(i, e)| {
                    // println!("at {},{}: {}", i, line_idx, mask.at(i, line_idx));
                    assert_eq!(mask.at(i, line_idx), *e == 1)
                });
            }
        }

        #[test]
        fn big_mask() {
            let c1 = ComputedWidgetPlacement {
                x: 0,
                y: 0,
                width: 40,
                height: 2,
            };
            let c2 = ComputedWidgetPlacement {
                x: 5,
                y: 1,
                width: 38,
                height: 2,
            };

            let mut mask = Mask::new_transparent(70, 4);
            mask.add(c1);
            mask.add(c2);

            mask.data
                .chunks(3)
                .for_each(|e| println!("{:032b} {:032b} {:032b}", e[0], e[1], e[2]));

            let expected = vec![
                0b00000000000000000000000000000000,
                0b00000000111111111111111111111111,
                0b11111111111111111111111111111111,
                0b00000000000000000000000000000000,
                0b00000000000111111111111111111111,
                0b11111111111111111111111111111111,
                0b11111000000000000000000000000000,
                0b00000000000111111111111111111111,
                0b11111111111111111111111111111111,
                0b11111111111111111111111111111111,
                0b11111111111111111111111111111111,
                0b11111111111111111111111111111111,
            ];
            assert_eq!(mask.data, expected);
        }
    }
}
