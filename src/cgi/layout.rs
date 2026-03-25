use std::collections::HashMap;

use crate::cgi::Displayable;
use crate::cgi::widget::{Widget, WidgetHdl};

#[derive(Debug)]
pub struct Layout {
    pub(crate) widgets: Vec<WidgetHdl>,
    last_widget: usize,
    names: HashMap<String, usize>,
    layout_data: SpacialTree<LayoutData>,
}

pub struct ComputedLayout(Vec<(WidgetHdl, LayoutData)>);

/// Node in a spacial tree, stores keys to its neighbors and its data
/// The nodes need to be stored in a structure that allows access by K
/// The data needs to be stored in a structure that allows access by V
#[derive(Debug, Clone)]
struct SpacialNode<K, V> {
    data_ref: V,
    left: Option<K>,
    right: Option<K>,
    top: Option<K>,
    bottom: Option<K>,
}

// Impl drop so that changes are applied at drop
pub struct PlacementOptions<'a> {
    parent: &'a mut Layout,
    widget_ref: usize,
    parent_ref: Option<usize>,
    coords: Option<(Coordinate, Coordinate)>,
    width: Option<Coordinate>,
    height: Option<Coordinate>,
    placement: Option<LayoutConstraint>,
    name: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Coordinate {
    Absolute(i32),
    Relative(f32),
    Hybrid(i32, f32),
    Adaptative,
}

impl Coordinate {
    fn is_null(&self) -> bool {
        match self {
            Coordinate::Absolute(a) => *a == 0,
            Coordinate::Relative(r) => *r == 0.0,
            Coordinate::Hybrid(a, r) => *a == 0 && *r == 0.0,
            Coordinate::Adaptative => false,
        }
    }

    fn compute_adaptative_sizes(coords: &[Self]) -> Self {
        let mut space_to_occupy = Self::Hybrid(0, 1.0); // full size
        let mut divider = 0;
        for coord in coords {
            if let Coordinate::Adaptative = coord {
                divider += 1;
            } else {
                space_to_occupy = space_to_occupy - *coord;
            }
        }
        if divider == 0 {
            return Self::Hybrid(0, 0.0);
        }
        let Coordinate::Hybrid(a, r) = space_to_occupy else {
            unreachable!()
        };
        let relative = r / divider as f32;
        let absolute = a / divider as i32;
        return Self::Hybrid(absolute, relative);
    }
}

impl std::ops::Add for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Self) -> Self::Output {
        use Coordinate::*;

        if self.is_null() {
            return rhs;
        }
        if rhs.is_null() {
            return self;
        }
        match (self, rhs) {
            (Absolute(a1), Absolute(a2)) => Absolute(a1 + a2),
            (Relative(r1), Relative(r2)) => Relative(r1 + r2),
            (Hybrid(a1, r1), Hybrid(a2, r2)) => Hybrid(a1 + a2, r1 + r2),
            (Absolute(a), Relative(r)) | (Relative(r), Absolute(a)) => Hybrid(a, r),
            (Absolute(a), Hybrid(b1, b2)) | (Hybrid(b1, b2), Absolute(a)) => Hybrid(a + b1, b2),
            (Relative(a), Hybrid(b1, b2)) | (Hybrid(b1, b2), Relative(a)) => Hybrid(b1, a + b2),
            (Adaptative, _) | (_, Adaptative) => Adaptative,
        }
    }
}

impl std::ops::Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, rhs: Self) -> Self::Output {
        use Coordinate::*;

        if rhs.is_null() {
            return self;
        }
        match (self, rhs) {
            (Absolute(a1), Absolute(a2)) => Absolute(a1 - a2),
            (Relative(r1), Relative(r2)) => Relative(r1 - r2),
            (Hybrid(a1, r1), Hybrid(a2, r2)) => Hybrid(a1 - a2, r1 - r2),
            (Absolute(a), Relative(r)) => Hybrid(a, -r),
            (Relative(r), Absolute(a)) => Hybrid(-a, r),
            (Absolute(a), Hybrid(a2, r2)) => Hybrid(a - a2, -r2),
            (Hybrid(a1, r1), Absolute(a2)) => Hybrid(a1 - a2, r1),
            (Relative(r1), Hybrid(a2, r2)) => Hybrid(-a2, r1 - r2),
            (Hybrid(a1, r1), Relative(r2)) => Hybrid(a1, r1 - r2),
            (Adaptative, _) | (_, Adaptative) => Adaptative,
        }
    }
}

#[derive(Debug)]
enum LayoutConstraint {
    Left(Coordinate),
    Right(Coordinate),
    Above(Coordinate),
    Below(Coordinate),
}

#[derive(Debug)]
struct LayoutData {
    tl: (Coordinate, Coordinate),
    width: Coordinate,
    height: Coordinate,
}

impl<K, V> SpacialNode<K, V> {
    fn new(data_ref: V) -> Self {
        Self {
            data_ref,
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

impl SpacialNode<usize, usize> {
    fn next_ordered<'a>(&self, data: &'a mut [usize; 4]) -> &'a [usize] {
        let default = 1000;
        let mut size = 0;
        data[0] = self.left.unwrap_or_else(|| {
            size += 1;
            default
        });
        data[1] = self.right.unwrap_or_else(|| {
            size += 1;
            default
        });
        data[2] = self.top.unwrap_or_else(|| {
            size += 1;
            default
        });
        data[3] = self.bottom.unwrap_or_else(|| {
            size += 1;
            default
        });
        data.sort();

        &data[..size]
    }
}

#[derive(Debug)]
struct SpacialTree<T> {
    nodes: Vec<SpacialNode<usize, usize>>,
    data: Vec<T>,
}

impl<T> SpacialTree<T> {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            data: Vec::new(),
        }
    }

    fn add_node(&mut self, data: T) -> usize {
        self.data.push(data);
        self.nodes.push(SpacialNode::new(self.data.len() - 1));
        self.data.len() - 1
    }

    fn last_node_data(&mut self) -> &mut T {
        self.data.last_mut().unwrap()
    }

    fn add_left(&mut self, data: T, node_ref: usize) -> usize {
        let new_node_ref = self.add_node(data);
        if let Some(node_between) = self.nodes[node_ref].left {
            self.nodes[new_node_ref].left = Some(node_between);
            self.nodes[node_between].right = Some(new_node_ref);
        }
        self.nodes[node_ref].left = Some(new_node_ref);
        self.nodes[new_node_ref].right = Some(node_ref);
        new_node_ref
    }

    fn add_right(&mut self, data: T, node_ref: usize) -> usize {
        let new_node_ref = self.add_node(data);
        if let Some(node_between) = self.nodes[node_ref].right {
            self.nodes[new_node_ref].right = Some(node_between);
            self.nodes[node_between].left = Some(new_node_ref);
        }
        self.nodes[node_ref].right = Some(new_node_ref);
        self.nodes[new_node_ref].left = Some(node_ref);
        new_node_ref
    }

    fn add_above(&mut self, data: T, node_ref: usize) -> usize {
        let new_node_ref = self.add_node(data);
        if let Some(node_between) = self.nodes[node_ref].top {
            self.nodes[new_node_ref].top = Some(node_between);
            self.nodes[node_between].bottom = Some(new_node_ref);
        }
        self.nodes[node_ref].top = Some(new_node_ref);
        self.nodes[new_node_ref].bottom = Some(node_ref);
        new_node_ref
    }

    fn add_below(&mut self, data: T, node_ref: usize) -> usize {
        let new_node_ref = self.add_node(data);
        if let Some(node_between) = self.nodes[node_ref].bottom {
            self.nodes[new_node_ref].bottom = Some(node_between);
            self.nodes[node_between].top = Some(new_node_ref);
        }
        self.nodes[node_ref].bottom = Some(new_node_ref);
        self.nodes[new_node_ref].top = Some(node_ref);
        new_node_ref
    }

    fn left_to_right(&self, mut of: usize) -> Vec<usize> {
        let mut result = Vec::new();

        while let Some(node_ref) = self.nodes[of].left {
            of = node_ref;
        }
        result.push(of);
        while let Some(node_ref) = self.nodes[of].right {
            result.push(node_ref);
            of = node_ref;
        }
        result
    }

    fn top_to_bottom(&self, mut of: usize) -> Vec<usize> {
        let mut result = Vec::new();

        while let Some(node_ref) = self.nodes[of].top {
            of = node_ref;
        }
        result.push(of);
        while let Some(node_ref) = self.nodes[of].bottom {
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
}

impl<T> std::ops::Index<usize> for SpacialTree<T> {
    type Output = SpacialNode<usize, usize>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> std::ops::Index<&SpacialNode<usize, usize>> for SpacialTree<T> {
    type Output = T;

    fn index(&self, index: &SpacialNode<usize, usize>) -> &Self::Output {
        &self.data[index.data_ref]
    }
}

impl LayoutData {
    fn new(x: Coordinate, y: Coordinate, width: Coordinate, height: Coordinate) -> Self {
        Self {
            tl: (x, y),
            width,
            height,
        }
    }

    fn empty() -> Self {
        Self {
            tl: (Coordinate::Adaptative, Coordinate::Adaptative),
            width: Coordinate::Adaptative,
            height: Coordinate::Adaptative,
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

impl Layout {
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

    fn compute(&self) -> ComputedLayout {
        let mut r = Vec::new();

        let node_idx = 0;
        let node = &self.layout_data[node_idx];
        if let Coordinate::Adaptative = self.layout_data[node].width {
            let line = self.layout_data.left_to_right(node_idx);
            let adaptative_size = Coordinate::compute_adaptative_sizes(
                &line
                    .iter()
                    .map(|e| self.layout_data.data[*e].width)
                    .collect::<Vec<_>>(),
            );

            for child in line.iter().map(|e| self.layout_data.data[*e]) {
                if let Coordinate::Adaptative = child.width {
                    child.width = adaptative_size;
                }
            }
        }
        let mut childs = [0; 4];
        for child in node.next_ordered(&mut childs) {}

        ComputedLayout(r)
    }
}

impl<'a> PlacementOptions<'a> {
    fn new(layout: &'a mut Layout) -> Self {
        Self {
            widget_ref: layout.layout_data.nodes.len(),
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
            // TODO: handle offsets
            let parent_ref = &self.parent.layout_data.data[self.parent_ref.unwrap()];
            match placement {
                LayoutConstraint::Left(offset) => {
                    let layout =
                        LayoutData::new(Adaptative, parent_ref.tl.1, Adaptative, parent_ref.height);
                    self.parent
                        .layout_data
                        .add_left(layout, self.parent_ref.unwrap());
                }
                LayoutConstraint::Right(offset) => {
                    let layout =
                        LayoutData::new(Adaptative, parent_ref.tl.1, Adaptative, parent_ref.height);
                    self.parent
                        .layout_data
                        .add_right(layout, self.parent_ref.unwrap());
                }
                LayoutConstraint::Above(offset) => {
                    let layout =
                        LayoutData::new(parent_ref.tl.0, Adaptative, parent_ref.width, Adaptative);
                    self.parent
                        .layout_data
                        .add_above(layout, self.parent_ref.unwrap());
                }
                LayoutConstraint::Below(offset) => {
                    let layout =
                        LayoutData::new(parent_ref.tl.0, Adaptative, parent_ref.width, Adaptative);
                    self.parent
                        .layout_data
                        .add_below(layout, self.parent_ref.unwrap());
                }
            }
        } else {
            // No relative placement specified
            self.parent.layout_data.add_node(LayoutData::empty());
        }
        let layout_data = &mut self.parent.layout_data.data[self.widget_ref];
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
        assert_eq!(Adaptative + Absolute(5), Adaptative);
        assert_eq!(Relative(0.2) + Adaptative, Adaptative);
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
            computed_layout: &ComputedLayout,
            widget_data: u32,
        ) -> Option<&LayoutData> {
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
            let mut layout = Layout::new();
            let mut dg = super::DummyGenerator::new();
            let widgets = dg.get_n_widgets(2);

            layout.add_widget(&widgets[0]);
            layout
                .add_widget(&widgets[1])
                .under_last_widget(Absolute(0));

            let computed_layout = layout.compute();
            let w1_layout_data = unsafe { get_widget(&computed_layout, 0) };
            let w2_layout_data = unsafe { get_widget(&computed_layout, 1) };

            println!("{:?} \n {:?}", w1_layout_data, w2_layout_data);
        }

        #[test]
        fn simple_layout() {
            let mut layout = Layout::new();
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

            layout.compute();
            //
        }
    }
}
