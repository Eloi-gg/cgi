use std::collections::{HashMap, HashSet};

use crate::cgi::Displayable;
use crate::cgi::widget::{Widget, WidgetHdl};

#[derive(Debug)]
pub struct Layout {
    pub(crate) widgets: Vec<WidgetHdl>,
    last_widget: usize,
    names: HashMap<String, usize>,
    layout_data: SpacialTree<LayoutData>,
}

pub struct ComputedLayout(HashMap<WidgetHdl, LayoutData>);

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
    parent: &'a mut Layout,
    widget_ref: usize,
    parent_ref: Option<usize>,
    coords: Option<(Coordinate, Coordinate)>,
    width: Option<Coordinate>,
    height: Option<Coordinate>,
    placement: Option<LayoutConstraint>,
    name: Option<String>,
}

#[derive(Debug, Copy, Clone)]
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


impl PartialEq for Coordinate {
    fn eq(&self, other: &Self) -> bool {
        use Coordinate::*;

        match (self, other) {
            (Absolute(a1), Absolute(a2)) => a1 == a2,
            (Relative(r1), Relative(r2)) => r1 == r2,
            (Hybrid(a1, r1), Hybrid(a2, r2)) => a1 == a2 && r1 == r2,
            (Adaptative, Adaptative) => true,
            
            (Absolute(0), Relative(0.0)) | (Relative(0.0), Absolute(0)) => true,
            (Absolute(0), Hybrid(0, 0.0)) | (Hybrid(0, 0.0), Absolute(0)) => true,

            (Absolute(a), Hybrid(a2, 0.0)) | (Hybrid(a2, 0.0), Absolute(a)) => a == a2,
            (Relative(r), Hybrid(0, r2)) | (Hybrid(0, r2), Relative(r)) => r == r2,

            (Adaptative, Relative(1.0)) | (Relative(1.0), Adaptative) => true,
            _ => false,
        }
    }
}
impl Eq for Coordinate {}

#[derive(Debug)]
enum LayoutConstraint {
    Left(Coordinate),
    Right(Coordinate),
    Above(Coordinate),
    Below(Coordinate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LayoutData {
    tl: (Coordinate, Coordinate),
    width: Coordinate,
    height: Coordinate,
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

        &data[..(4-missing_elts)]
    }
}

#[derive(Debug)]
struct SpacialTree<T> (Vec<SpacialNode<usize, T>>);

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

    fn top_left_node(&self) -> (&SpacialNode<usize, T>, usize) {
        let mut current = &self.0[0];
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

    fn compute_size_recursive(&mut self, node_ref: usize, computed_layout: &mut HashMap<WidgetHdl, LayoutData>) {
        use Coordinate::*;

        if computed_layout.contains_key(&self.widgets[node_ref]) {
            return;
        }

        let node = self.layout_data[node_ref].clone();
        // Compute width and height for adaptative widgets, then push them to the result vector in the correct order
        if let Adaptative = node.data.width {
            let line = self.layout_data.left_to_right(node_ref);
            let adaptative_size = Coordinate::compute_adaptative_sizes(
                &line
                    .iter()
                    .map(|e| self.layout_data.0[*e].data.width)
                    .collect::<Vec<_>>(),
            );

            for child_ref in line.iter() {
                let child_data = &mut self.layout_data.0[*child_ref].data;
                if let Adaptative = child_data.width {
                    child_data.width = adaptative_size;
                }
            }

            self.layout_data.0[node_ref].data.width = adaptative_size;
        } else if let Adaptative = node.data.height {
             let column = self.layout_data.top_to_bottom(node_ref);
            let adaptative_size = Coordinate::compute_adaptative_sizes(
                &column
                    .iter()
                    .map(|e| self.layout_data.0[*e].data.height)
                    .collect::<Vec<_>>(),
            );

            for child_ref in column.iter() {
                let child_data = &mut self.layout_data.0[*child_ref].data;
                if let Adaptative = child_data.height {
                    child_data.height = adaptative_size;
                } 
            }

            self.layout_data.0[node_ref].data.height = adaptative_size;
        } 

        computed_layout.insert(self.widgets[node_ref].clone(), self.layout_data.0[node_ref].data.clone());

        for child in node.next_ordered(&mut [0; 4]) {
            self.compute_size_recursive(*child, computed_layout);
        } 
    }

    fn compute_coords_recursive(&self, node_ref: usize, computed_layout: &mut HashMap<WidgetHdl, LayoutData>) {
        use Coordinate::*;

        let node = &self.layout_data[node_ref];
        let wref = &self.widgets[node_ref];
        let node_layout = computed_layout[wref].clone();

        for child_ref in node.next_ordered(&mut [0; 4]) {
            let child_layout = computed_layout.get_mut(&self.widgets[*child_ref]).unwrap();
            // TODO: handle offsets
            if let Adaptative = child_layout.tl.0 {
                child_layout.tl.0 = node_layout.tl.0 + node_layout.width;
            }
            if let Adaptative = child_layout.tl.1 {
                child_layout.tl.1 = node_layout.tl.1 + node_layout.height;
            }
            self.compute_coords_recursive(*child_ref, computed_layout);
        } 
    }

    fn compute(mut self) -> ComputedLayout {
        use Coordinate::*;

        let mut r: HashMap<WidgetHdl, LayoutData> = HashMap::new();
        let mut starting_points = Vec::new();

        
        for node_idx in 0..self.layout_data.0.len() {
            if !r.contains_key(&self.widgets[node_idx]) {
                starting_points.push(node_idx);
            }
            self.compute_size_recursive(node_idx, &mut r);
        }
        // The widget on the top left corner gets 0,0 as coordinate if no coordinate are set
        let top_left_node = self.layout_data.top_left_node().1;
        if let (Adaptative, Adaptative) = r[&self.widgets[top_left_node]].tl {
            r.insert(self.widgets[top_left_node].clone(), r[&self.widgets[top_left_node]].clone().with_coords(Absolute(0), Absolute(0)));
        }
        
        for node_idx in starting_points {
            self.compute_coords_recursive(node_idx, &mut r);
        }

        ComputedLayout(r)
    }
}

impl<'a> PlacementOptions<'a> {
    fn new(layout: &'a mut Layout) -> Self {
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
            // TODO: handle offsets
            let parent_ref = &self.parent.layout_data.0[self.parent_ref.unwrap()].data;
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

            assert_eq!(layout.layout_data.top_left_node().1, 0); // The first widget added should be the top left node
            
            layout.compute();

            //
        }
    }
}
