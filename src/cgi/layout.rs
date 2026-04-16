use crate::cgi::Displayable;
use crate::cgi::coordinate::Coordinate;
use crate::cgi::widget::{Widget, WidgetHdl};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct LayoutBuilder {
    pub(crate) widgets: Vec<WidgetHdl>,
    last_widget: usize,
    names: HashMap<String, usize>,
    layout_data: SpacialTree<WidgetPlacement>,
}

pub struct Layout(HashMap<WidgetHdl, WidgetPlacement>);
pub(crate) struct ComputedLayout(HashMap<WidgetHdl, ComputedWidgetPlacement>);

/// Node in a spacial tree, stores keys to its neighbors and its data
/// The nodes need to be stored in a structure that allows access by K
#[derive(Debug, Clone)]
struct SpacialNode<K, V> {
    data: V,
    left: Option<K>,
    right: Option<K>,
    top: Option<K>,
    bottom: Option<K>,
}

// Impl drop so that changes are applied at drop
pub struct PlacementOptions<'a> {
    parent: &'a mut LayoutBuilder,
    widget_ref: usize,
    parent_ref: Option<usize>,
    coords: Option<(Coordinate, Coordinate)>,
    width: Option<Coordinate>,
    height: Option<Coordinate>,
    placement: Option<LayoutConstraint>,
    name: Option<String>,
}

#[derive(Debug)]
enum LayoutConstraint {
    Left(Coordinate),
    Right(Coordinate),
    Above(Coordinate),
    Below(Coordinate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WidgetPlacement {
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

impl<K, V> SpacialNode<K, V> {
    fn new(data: V) -> Self {
        Self {
            data,
            left: None,
            right: None,
            top: None,
            bottom: None,
        }
    }

    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none() && self.top.is_none() && self.bottom.is_none()
    }
}

impl<T> SpacialNode<usize, T> {
    /// Returns the keys of the neighbors of the node in the following order : left, right, top, bottom.
    fn next_ordered<'a>(&self, data: &'a mut [usize; 4]) -> &'a [usize] {
        let default = 1000;
        let mut missing_elts = 0;
        data[0] = self.left.unwrap_or_else(|| {
            missing_elts += 1;
            default
        });
        data[1] = self.right.unwrap_or_else(|| {
            missing_elts += 1;
            default
        });
        data[2] = self.top.unwrap_or_else(|| {
            missing_elts += 1;
            default
        });
        data[3] = self.bottom.unwrap_or_else(|| {
            missing_elts += 1;
            default
        });
        data.sort();

        &data[..(4 - missing_elts)]
    }
}

#[derive(Debug)]
struct SpacialTree<T>(Vec<SpacialNode<usize, T>>);

impl<T> SpacialTree<T> {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add_node(&mut self, data: T) -> usize {
        self.0.push(SpacialNode::new(data));
        self.0.len() - 1
    }

    fn last_node_data(&mut self) -> &mut T {
        &mut self.0.last_mut().unwrap().data
    }

    fn add_left(&mut self, data: T, node_ref: usize) -> usize {
        let new_node_ref = self.add_node(data);
        if let Some(node_between) = self.0[node_ref].left {
            self.0[new_node_ref].left = Some(node_between);
            self.0[node_between].right = Some(new_node_ref);
        }
        self.0[node_ref].left = Some(new_node_ref);
        self.0[new_node_ref].right = Some(node_ref);
        new_node_ref
    }

    fn add_right(&mut self, data: T, node_ref: usize) -> usize {
        let new_node_ref = self.add_node(data);
        if let Some(node_between) = self.0[node_ref].right {
            self.0[new_node_ref].right = Some(node_between);
            self.0[node_between].left = Some(new_node_ref);
        }
        self.0[node_ref].right = Some(new_node_ref);
        self.0[new_node_ref].left = Some(node_ref);
        new_node_ref
    }

    fn add_above(&mut self, data: T, node_ref: usize) -> usize {
        let new_node_ref = self.add_node(data);
        if let Some(node_between) = self.0[node_ref].top {
            self.0[new_node_ref].top = Some(node_between);
            self.0[node_between].bottom = Some(new_node_ref);
        }
        self.0[node_ref].top = Some(new_node_ref);
        self.0[new_node_ref].bottom = Some(node_ref);
        new_node_ref
    }

    fn add_below(&mut self, data: T, node_ref: usize) -> usize {
        let new_node_ref = self.add_node(data);
        if let Some(node_between) = self.0[node_ref].bottom {
            self.0[new_node_ref].bottom = Some(node_between);
            self.0[node_between].top = Some(new_node_ref);
        }
        self.0[node_ref].bottom = Some(new_node_ref);
        self.0[new_node_ref].top = Some(node_ref);
        new_node_ref
    }

    fn left_to_right(&self, mut of: usize) -> Vec<usize> {
        let mut result = Vec::new();

        while let Some(node_ref) = self.0[of].left {
            of = node_ref;
        }
        result.push(of);
        while let Some(node_ref) = self.0[of].right {
            result.push(node_ref);
            of = node_ref;
        }
        result
    }

    fn top_to_bottom(&self, mut of: usize) -> Vec<usize> {
        let mut result = Vec::new();

        while let Some(node_ref) = self.0[of].top {
            of = node_ref;
        }
        result.push(of);
        while let Some(node_ref) = self.0[of].bottom {
            result.push(node_ref);
            of = node_ref;
        }
        result
    }

    // TODO : delete
    fn ordered_tree(&self, root: usize) -> Vec<usize> {
        let mut result = Vec::new();
        for top in self.top_to_bottom(root) {
            for left in self.left_to_right(top) {
                result.push(left);
            }
        }
        result
    }

    fn top_left_from(&self, idx: usize) -> (&SpacialNode<usize, T>, usize) {
        let mut current = &self.0[idx];
        let mut current_ref = 0;

        loop {
            if let Some(top) = current.top {
                current = &self.0[top];
                current_ref = top;
            } else if let Some(left) = current.left {
                current = &self.0[left];
                current_ref = left;
            } else {
                break;
            }
        }
        (&current, current_ref)
    }
}

impl<T> std::ops::Index<usize> for SpacialTree<T> {
    type Output = SpacialNode<usize, T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl WidgetPlacement {
    fn new(x: Coordinate, y: Coordinate, width: Coordinate, height: Coordinate) -> Self {
        Self {
            tl: (x, y),
            width,
            height,
        }
    }

    fn empty() -> Self {
        Self {
            tl: (Coordinate::Adaptative(0), Coordinate::Adaptative(0)),
            width: Coordinate::Adaptative(0),
            height: Coordinate::Adaptative(0),
        }
    }

    fn with_width(self, width: Coordinate) -> Self {
        Self { width, ..self }
    }

    fn with_height(self, height: Coordinate) -> Self {
        Self { height, ..self }
    }

    fn with_coords(self, x: Coordinate, y: Coordinate) -> Self {
        Self { tl: (x, y), ..self }
    }
}

impl LayoutBuilder {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            last_widget: 0, // Todo remove
            layout_data: SpacialTree::new(),
            names: HashMap::new(),
        }
    }

    pub fn add_widget<D: Displayable + 'static>(&mut self, widget: &Widget<D>) -> PlacementOptions {
        self.widgets.push(WidgetHdl {
            widget: widget.as_dyn(),
        });

        PlacementOptions::new(self)
    }

    fn compute_size_recursive(
        &mut self,
        node_ref: usize,
        computed_layout: &mut HashMap<WidgetHdl, WidgetPlacement>,
    ) {
        use Coordinate::*;

        if computed_layout.contains_key(&self.widgets[node_ref]) {
            return;
        }

        let node = self.layout_data[node_ref].clone();
        // Compute width and height for adaptative widgets, then push them to the result vector in the correct order
        if let Adaptative(offset) = node.data.width {
            let line = self.layout_data.left_to_right(node_ref);
            let last_offset = self.layout_data.0[*line.last().unwrap()]
                .data
                .tl
                .0
                .absolute_part_i32();
            let adaptative_size = Coordinate::compute_adaptative_sizes(
                &line
                    .iter()
                    .map(|e| self.layout_data.0[*e].data.width)
                    .collect::<Vec<_>>(),
                -last_offset,
            );

            for child_ref in line.iter() {
                let child_data = &mut self.layout_data.0[*child_ref].data;
                if let Adaptative(offset) = child_data.width {
                    child_data.width = adaptative_size + Absolute(offset);
                }
            }

            self.layout_data.0[node_ref].data.width = adaptative_size + Absolute(offset);
        }
        if let Adaptative(offset) = node.data.height {
            let column = self.layout_data.top_to_bottom(node_ref);
            let last_offset = self.layout_data.0[*column.last().unwrap()]
                .data
                .tl
                .1
                .absolute_part_i32();
            let adaptative_size = Coordinate::compute_adaptative_sizes(
                &column
                    .iter()
                    .map(|e| self.layout_data.0[*e].data.height)
                    .collect::<Vec<_>>(),
                -last_offset,
            );

            for child_ref in column.iter() {
                let child_data = &mut self.layout_data.0[*child_ref].data;
                if let Adaptative(offset) = child_data.height {
                    child_data.height = adaptative_size + Absolute(offset);
                }
            }

            self.layout_data.0[node_ref].data.height = adaptative_size + Absolute(offset);
        }

        computed_layout.insert(
            self.widgets[node_ref].clone(),
            self.layout_data.0[node_ref].data.clone(),
        );

        for child in node.next_ordered(&mut [0; 4]) {
            self.compute_size_recursive(*child, computed_layout);
        }
    }

    // Coords should be (0,0) at call, out should be an empty hashmap
    // Expected to be called on top-left node
    // TODO : keep private and give better interface
    fn compute_offsets_recursive(
        &self,
        node_ref: usize,
        coords: (u16, u16),
        out: &mut HashMap<(u16, u16), (i32, i32)>,
    ) {
        let node = &self.layout_data[node_ref];

        if out.contains_key(&coords) {
            return;
        }

        let total_offset_x = self
            .layout_data
            .left_to_right(node_ref)
            .into_iter()
            .map(|e| &self.layout_data[e])
            .fold(0, |acc, e| {
                acc + e.data.tl.0.absolute_part_i32() + e.data.width.absolute_part_i32()
            });

        let total_offset_y = self
            .layout_data
            .top_to_bottom(node_ref)
            .into_iter()
            .map(|e| &self.layout_data[e])
            .fold(0, |acc, e| {
                acc + e.data.tl.1.absolute_part_i32() + e.data.height.absolute_part_i32()
            });

        out.insert(coords, (total_offset_x, total_offset_y));
        if let Some(child_ref) = node.right {
            self.compute_offsets_recursive(child_ref, (coords.0 + 1, coords.1), out);
        }
        if let Some(child_ref) = node.bottom {
            self.compute_offsets_recursive(child_ref, (coords.0, coords.1 + 1), out);
        }
        

        // all directions...
    }

    /// Computes the coordinates of the widgets in the layout. Expected to be called on a top-left node.
    /// It will not look at upper or left neighbors.
    fn compute_coords_recursive(
        &self,
        node_ref: usize,
        computed_layout: &mut HashMap<WidgetHdl, WidgetPlacement>,
    ) {
        use Coordinate::*;

        let node = &self.layout_data[node_ref];
        let wref = &self.widgets[node_ref];
        let node_layout = computed_layout[wref].clone();

        if let Some(child_ref) = node.right {
            let child_layout = computed_layout.get_mut(&self.widgets[child_ref]).unwrap();

            child_layout.tl.0 += node_layout.tl.0 + node_layout.width;

            self.compute_coords_recursive(child_ref, computed_layout);
        }
        if let Some(child_ref) = node.bottom {
            let child_layout = computed_layout.get_mut(&self.widgets[child_ref]).unwrap();

            child_layout.tl.1 += node_layout.tl.1 + node_layout.height;

            self.compute_coords_recursive(child_ref, computed_layout);
        }
    }

    fn process(mut self) -> Layout {
        use Coordinate::*;

        let mut r: HashMap<WidgetHdl, WidgetPlacement> = HashMap::new();
        let mut starting_points = Vec::new();

        for node_idx in 0..self.layout_data.0.len() {
            if !r.contains_key(&self.widgets[node_idx]) {
                starting_points.push(node_idx);
            }
            self.compute_size_recursive(node_idx, &mut r);
        }
        // The widget on the top left corner gets 0,0 as coordinate if no coordinate are set
        let top_left_node = self.layout_data.top_left_from(0).1;
        if let (Adaptative(offset_x), Adaptative(offset_y)) = r[&self.widgets[top_left_node]].tl {
            r.insert(
                self.widgets[top_left_node].clone(),
                r[&self.widgets[top_left_node]]
                    .clone()
                    .with_coords(Absolute(offset_x), Absolute(offset_y)),
            );
        }

        for node_idx in starting_points {
            self.compute_coords_recursive(self.layout_data.top_left_from(node_idx).1, &mut r);
        }

        Layout(r)
    }
}

impl Layout {
    // fn compute(self, size_x: u32, size_y: u32) -> ComputedLayout {
    //     ComputedLayout(self.0.into_iter().map(|(k,v)| (k, v.)))
    // }
}

impl<'a> PlacementOptions<'a> {
    fn new(layout: &'a mut LayoutBuilder) -> Self {
        Self {
            widget_ref: layout.layout_data.0.len(),
            parent: layout,
            parent_ref: None,
            coords: None,
            width: None,
            height: None,
            placement: None,
            name: None,
        }
    }

    pub fn at_coords(mut self, x: Coordinate, y: Coordinate) -> Self {
        self.coords = Some((x, y));
        self
    }

    pub fn right_to_last_widget(mut self, offset: Coordinate) -> Self {
        self.placement = Some(LayoutConstraint::Right(offset));
        self.parent_ref = Some(self.widget_ref - 1);
        self
    }

    pub fn under_last_widget(mut self, offset: Coordinate) -> Self {
        self.placement = Some(LayoutConstraint::Below(offset));
        self.parent_ref = Some(self.widget_ref - 1);
        self
    }

    pub fn above_last_widget(mut self, offset: Coordinate) -> Self {
        self.placement = Some(LayoutConstraint::Above(offset));
        self.parent_ref = Some(self.widget_ref - 1);
        self
    }

    pub fn left_to_last_widget(mut self, offset: Coordinate) -> Self {
        self.placement = Some(LayoutConstraint::Left(offset));
        self.parent_ref = Some(self.widget_ref - 1);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_size(mut self, width: Coordinate, height: Coordinate) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_height(mut self, height: Coordinate) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_width(mut self, width: Coordinate) -> Self {
        self.width = Some(width);
        self
    }

    // TODO: add placement options relative to specified widget, not only the last one
}

impl Drop for PlacementOptions<'_> {
    fn drop(&mut self) {
        use Coordinate::*;

        if let Some(placement) = &self.placement {
            let parent_ref = &self.parent.layout_data.0[self.parent_ref.unwrap()].data;
            match placement {
                LayoutConstraint::Left(offset) => {
                    let layout = WidgetPlacement::new(
                        -*offset,
                        Absolute(0),
                        Adaptative(0),
                        parent_ref.height,
                    );
                    self.parent
                        .layout_data
                        .add_left(layout, self.parent_ref.unwrap());
                }
                LayoutConstraint::Right(offset) => {
                    let layout = WidgetPlacement::new(
                        *offset,
                        Absolute(0),
                        Adaptative(0),
                        parent_ref.height,
                    );
                    self.parent
                        .layout_data
                        .add_right(layout, self.parent_ref.unwrap());
                }
                LayoutConstraint::Above(offset) => {
                    let layout =
                        WidgetPlacement::new(Absolute(0), *offset, parent_ref.width, Adaptative(0));
                    self.parent
                        .layout_data
                        .add_above(layout, self.parent_ref.unwrap());
                }
                LayoutConstraint::Below(offset) => {
                    let layout =
                        WidgetPlacement::new(Absolute(0), *offset, parent_ref.width, Adaptative(0));
                    self.parent
                        .layout_data
                        .add_below(layout, self.parent_ref.unwrap());
                }
            }
        } else {
            // No relative placement specified
            self.parent.layout_data.add_node(WidgetPlacement::empty());
        }
        let layout_data = &mut self.parent.layout_data.0[self.widget_ref].data;
        if let Some(name) = &self.name {
            self.parent.names.insert(name.clone(), self.widget_ref);
        }
        if let Some((x, y)) = self.coords {
            layout_data.tl = (x, y);
        }
        if let Some(width) = self.width {
            layout_data.width = width;
        }
        if let Some(height) = self.height {
            layout_data.height = height;
        }
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

    mod tree {
        use super::super::*;

        #[test]
        fn spacial_tree_ordering() {
            let mut dg = super::DummyGenerator::new();
            {
                let mut widgets = dg.get_n_widgets(4).into_iter();
                let mut tree = SpacialTree::new();
                let root = tree.add_node(widgets.next().unwrap().as_dyn());
                let w1 = tree.add_left(widgets.next().unwrap().as_dyn(), root); // w1 is left of root
                let w2 = tree.add_right(widgets.next().unwrap().as_dyn(), w1); // w2 is right of w1, so w2 is between w1 and root
                let w3 = tree.add_right(widgets.next().unwrap().as_dyn(), root); // w3 is right of root

                // Expected order: w1, w2, root, w3
                let expected_order = vec![w1, w2, root, w3];
                let actual_order = tree.left_to_right(root);

                assert_eq!(actual_order, expected_order, "LR ordering failed");
            }

            {
                let mut widgets = dg.get_n_widgets(4).into_iter();
                let mut tree = SpacialTree::new();
                let root = tree.add_node(widgets.next().unwrap().as_dyn());
                let w1 = tree.add_above(widgets.next().unwrap().as_dyn(), root); // w1 is above root
                let w2 = tree.add_below(widgets.next().unwrap().as_dyn(), w1); // w2 is below w1, so w2 is between w1 and root
                let w3 = tree.add_below(widgets.next().unwrap().as_dyn(), root); // w3 is below root

                // Expected order: w1, w2, root, w3
                let expected_order = vec![w1, w2, root, w3];
                let actual_order = tree.top_to_bottom(root);

                assert_eq!(actual_order, expected_order, "TB ordering failed");
            }
        }

        #[test]
        fn full_tree_ordering() {
            let mut dg = super::DummyGenerator::new();
            let mut widgets = dg.get_n_widgets(6).into_iter();
            let mut tree = SpacialTree::new();
            let root = tree.add_node(widgets.next().unwrap().as_dyn());
            let w1 = tree.add_left(widgets.next().unwrap().as_dyn(), root); // w1 is left of root
            let w2 = tree.add_right(widgets.next().unwrap().as_dyn(), w1); // w2 is right of w1, so w2 is between w1 and root
            let w3 = tree.add_right(widgets.next().unwrap().as_dyn(), root); // w3 is right of root
            let w4 = tree.add_above(widgets.next().unwrap().as_dyn(), root); // w4 is above root
            let w5 = tree.add_below(widgets.next().unwrap().as_dyn(), root); // w5 is below root

            // Expected order: w4, w1, w2, root, w3, w5
            let expected_order = vec![w4, w1, w2, root, w3, w5];
            let actual_order = tree.ordered_tree(root);

            assert_eq!(actual_order, expected_order, "Full tree ordering failed");
        }
    }

    mod layout {
        use super::super::*;
        use super::Coordinate::*;

        // Looks through the computed_layout to find a Dummy widget with its data matching widget_data
        // ComputedLayout MUST contain only Dummy
        unsafe fn get_widget(
            computed_layout: &Layout,
            widget_data: u32,
        ) -> Option<&WidgetPlacement> {
            for (widget_hdl, layout_data) in &computed_layout.0 {
                let r = widget_hdl.widget.displayable.read().unwrap();
                let x = &*r as *const dyn Displayable as *const tests::Dummy;
                let current_widget_data = unsafe { (*x).data };
                {
                    if current_widget_data == widget_data {
                        return Some(layout_data);
                    }
                }
            }
            None
        }

        #[test]
        fn above_below() {
            let mut layout = LayoutBuilder::new();
            let mut dg = super::DummyGenerator::new();
            let widgets = dg.get_n_widgets(2);

            layout.add_widget(&widgets[0]);
            layout
                .add_widget(&widgets[1])
                .under_last_widget(Absolute(0));

            let computed_layout = layout.process();
            let w1_layout_data = unsafe { get_widget(&computed_layout, 0) };
            let w2_layout_data = unsafe { get_widget(&computed_layout, 1) };

            assert_eq!(w1_layout_data.unwrap().tl.0, Absolute(0));
            assert_eq!(w1_layout_data.unwrap().tl.1, Absolute(0));
            assert_eq!(w1_layout_data.unwrap().width, Relative(1.0));
            assert_eq!(w1_layout_data.unwrap().height, Relative(0.5));
            assert_eq!(w2_layout_data.unwrap().tl.0, Absolute(0));
            assert_eq!(w2_layout_data.unwrap().tl.1, Relative(0.5));
            assert_eq!(w2_layout_data.unwrap().width, Relative(1.0));
            assert_eq!(w2_layout_data.unwrap().height, Relative(0.5));
        }

        #[test]
        fn left_right() {
            let mut layout = LayoutBuilder::new();
            let mut dg = super::DummyGenerator::new();
            let widgets = dg.get_n_widgets(2);

            layout.add_widget(&widgets[0]);
            layout
                .add_widget(&widgets[1])
                .right_to_last_widget(Absolute(5));

            let computed_layout = layout.process();
            let w1_layout_data = unsafe { get_widget(&computed_layout, 0) };
            let w2_layout_data = unsafe { get_widget(&computed_layout, 1) };

            assert_eq!(w1_layout_data.unwrap().tl.0, Absolute(0));
            assert_eq!(w1_layout_data.unwrap().tl.1, Absolute(0));
            assert_eq!(w1_layout_data.unwrap().width, Relative(0.5));
            assert_eq!(w1_layout_data.unwrap().height, Relative(1.0));

            assert_eq!(w2_layout_data.unwrap().tl.0, Hybrid(5 / 2, 0.5));
            assert_eq!(w2_layout_data.unwrap().tl.1, Absolute(0));
            assert_eq!(w2_layout_data.unwrap().width, Relative(0.5));
            assert_eq!(w2_layout_data.unwrap().height, Relative(1.0));
        }

        #[test]
        fn simple_layout() {
            let mut layout = LayoutBuilder::new();
            let mut dg = super::DummyGenerator::new();
            let widgets = dg.get_n_widgets(4);

            layout
                .add_widget(&widgets[0])
                .at_coords(Absolute(5), Absolute(5))
                .with_size(Relative(0.3), Relative(0.3));
            layout
                .add_widget(&widgets[1])
                .right_to_last_widget(Absolute(5));
            layout
                .add_widget(&widgets[2])
                .under_last_widget(Absolute(5));
            layout
                .add_widget(&widgets[3])
                .under_last_widget(Absolute(5))
                .with_height(Absolute(20));

            assert_eq!(layout.layout_data.top_left_from(0).1, 0); // The first widget added should be the top left node

            let computed_layout = layout.process();

            let w1_layout_data = unsafe { get_widget(&computed_layout, 0).unwrap() };
            let w2_layout_data = unsafe { get_widget(&computed_layout, 1).unwrap() };
            let w3_layout_data = unsafe { get_widget(&computed_layout, 2).unwrap() };
            let w4_layout_data = unsafe { get_widget(&computed_layout, 3).unwrap() };

            // TODO: write actual assertions

            // First widget
            assert_eq!(w1_layout_data.tl, (Absolute(5), Absolute(5)));
            assert_eq!(w1_layout_data.width, Relative(0.3));
            assert_eq!(w1_layout_data.height, Relative(0.3));

            // Second widget
            assert_eq!(w2_layout_data.tl, (Hybrid(10, 0.3), Absolute(5)));
            assert_eq!(w2_layout_data.width, Relative(0.7));
            assert_eq!(w2_layout_data.height, Relative(0.3));

            // Third widget
            assert_eq!(w3_layout_data.tl, (Hybrid(10, 0.3), Hybrid(10, 0.3)));
            assert_eq!(w3_layout_data.width, Relative(1.0));
            assert_eq!(w3_layout_data.height, Relative(0.7));

            // Fourth widget
            assert_eq!(w4_layout_data.tl, (Hybrid(10, 0.3), Hybrid(-20, 1.0)));
            assert_eq!(w4_layout_data.width, Relative(0.7));
            assert_eq!(w4_layout_data.height, Absolute(20));

            //
        }

        #[test]
        fn hybrid_sizes() {
            todo!("Here a test where sizes are all given hybrid. Important")
        }
    }
}
