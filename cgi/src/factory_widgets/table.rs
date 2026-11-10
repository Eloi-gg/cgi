use crate::{
    Displayable,
    text_formatting::{CombinedFormat, Format, attributes, colors},
};

use super::*;
pub trait Cell: Send + Sync + Clone {
    fn get_width(&self) -> u16;
    fn get_chars(&self, buffer: &mut Vec<char>);
}

pub trait ColumnType: Send + Sync + Clone {
    type CellType: Cell;

    fn default_cell(&self) -> Self::CellType;
}

/// Used to modify the content of a cell, given a value
/// The behaviour can change depending on the column type
/// For example, a float could change the value of a Cell of type float, or
/// it could change the amount of a progress bar.
pub trait IntoCell<C: ColumnType> {
    fn into_cell(&self, column: &C, cell: &mut C::CellType);
}

#[derive(Clone)]
enum StandardCellEnum {
    Text {
        text: String,
    },
    Integer {
        value: i32,
        as_str: String,
    },
    Float {
        value: f32,
        min_precision: u8,
        max_precision: u8,
        as_str: String,
    },
    Boolean {
        value: bool,
    },
    ProgressBar {
        amt: f32,
    },
}

#[derive(Clone)]
pub struct StandardCell(StandardCellEnum);

#[derive(Clone)]
pub enum StandardColumnType {
    Text {
        align: text::TextAlign,
    },
    Number {
        align: text::TextAlign,
    },
    Boolean {
        align: text::TextAlign,
    },
    ProgressBar {
        bar_type: progression::ProgressBarType,
    },
}

impl StandardColumnType {
    pub fn new_text(align: text::TextAlign) -> Self {
        StandardColumnType::Text { align }
    }

    pub fn new_number(align: text::TextAlign) -> Self {
        StandardColumnType::Number { align }
    }

    pub fn new_boolean(align: text::TextAlign) -> Self {
        StandardColumnType::Boolean { align }
    }

    pub fn new_progress_bar(bar_type: progression::ProgressBarType) -> Self {
        StandardColumnType::ProgressBar { bar_type }
    }
}

impl IntoCell<StandardColumnType> for f32 {
    fn into_cell(
        &self,
        column: &StandardColumnType,
        cell: &mut <StandardColumnType as ColumnType>::CellType,
    ) {
        if let (StandardColumnType::ProgressBar { .. }, StandardCellEnum::ProgressBar { amt }) =
            (column, &mut cell.0)
        {
            *amt = *self;
        } else if let (
            StandardColumnType::Number { .. },
            StandardCellEnum::Float { value, as_str, .. },
        ) = (column, &mut cell.0)
        {
            *value = *self;
            *as_str = self.to_string();
        }
    }
}

impl IntoCell<StandardColumnType> for i32 {
    fn into_cell(
        &self,
        column: &StandardColumnType,
        cell: &mut <StandardColumnType as ColumnType>::CellType,
    ) {
        if let (StandardColumnType::Number { .. }, StandardCellEnum::Integer { value, as_str }) =
            (column, &mut cell.0)
        {
            *value = *self;
            *as_str = self.to_string();
        }
    }
}

impl IntoCell<StandardColumnType> for &str {
    fn into_cell(
        &self,
        column: &StandardColumnType,
        cell: &mut <StandardColumnType as ColumnType>::CellType,
    ) {
        if let (StandardColumnType::Text { .. }, StandardCellEnum::Text { text }) =
            (column, &mut cell.0)
        {
            *text = (*self).to_owned();
        }
    }
}

impl IntoCell<StandardColumnType> for bool {
    fn into_cell(
        &self,
        column: &StandardColumnType,
        cell: &mut <StandardColumnType as ColumnType>::CellType,
    ) {
        if let (StandardColumnType::Boolean { .. }, StandardCellEnum::Boolean { value }) =
            (column, &mut cell.0)
        {
            *value = *self;
        }
    }
}

impl StandardCell {
    pub fn new_text(text: String) -> Self {
        Self(StandardCellEnum::Text { text })
    }

    pub fn new_integer(value: i32) -> Self {
        Self(StandardCellEnum::Integer {
            value,
            as_str: value.to_string(),
        })
    }

    pub fn new_float(value: f32, min_precision: u8, max_precision: u8) -> Self {
        let as_str = format!("{:.*}", max_precision as usize, value);
        Self(StandardCellEnum::Float {
            value,
            min_precision,
            max_precision,
            as_str,
        })
    }

    pub fn new_boolean(value: bool) -> Self {
        Self(StandardCellEnum::Boolean { value })
    }
}

impl Cell for StandardCell {
    fn get_width(&self) -> u16 {
        let w = match &self.0 {
            StandardCellEnum::Text { text } => text.len(),
            StandardCellEnum::Integer { as_str, .. } => as_str.len(),
            StandardCellEnum::Float { as_str, .. } => as_str.len(),
            StandardCellEnum::Boolean { value } => value.to_string().len(),
            StandardCellEnum::ProgressBar { .. } => 0, // 0 means fill width
        };

        w as u16
    }

    fn get_chars(&self, buffer: &mut Vec<char>) {
        let str = match &self.0 {
            StandardCellEnum::Text { text } => text,
            StandardCellEnum::Integer { as_str, .. } => as_str,
            StandardCellEnum::Float { as_str, .. } => as_str,
            StandardCellEnum::Boolean { value } => {
                if *value {
                    "true"
                } else {
                    "false"
                }
            }
            StandardCellEnum::ProgressBar { amt } => "TODO",
        };

        buffer.extend(str.chars());
    }
}

impl ColumnType for StandardColumnType {
    type CellType = StandardCell;

    fn default_cell(&self) -> Self::CellType {
        let e = match self {
            StandardColumnType::Text { .. } => StandardCellEnum::Text {
                text: String::new(),
            },
            StandardColumnType::Number { .. } => StandardCellEnum::Integer {
                value: 0,
                as_str: "0".to_string(),
            },
            StandardColumnType::Boolean { .. } => StandardCellEnum::Boolean { value: false },
            StandardColumnType::ProgressBar { .. } => StandardCellEnum::ProgressBar { amt: 0.0 },
        };
        StandardCell(e)
    }
}

pub struct StyleSet {
    even: CombinedFormat,
    odd: CombinedFormat,
    titles: CombinedFormat,
}

impl StyleSet {
    pub fn new() -> Self {
        Self {
            titles: colors::front::BLACK | colors::back::DARKGREEN | attributes::BOLD,
            even: colors::front::grey(15) | colors::back::grey(200),
            odd: colors::front::grey(10) | colors::back::grey(150),
        }
    }

    pub fn with_titles_format<T: Into<CombinedFormat>>(mut self, format: T) -> Self {
        self.titles = format.into();
        self
    }

    pub fn with_even_row_format<T: Into<CombinedFormat>>(mut self, format: T) -> Self {
        self.even = format.into();
        self
    }

    pub fn with_odd_row_format<T: Into<CombinedFormat>>(mut self, format: T) -> Self {
        self.odd = format.into();
        self
    }
}

pub struct Table<T: ColumnType> {
    titles: Vec<String>,
    columns: Vec<T>,
    column_widths: Vec<ColumnWidth>,
    cells: Vec<T::CellType>,
    computed_column_widths: Vec<u16>,
    max_rows: u16,
    current_num_rows: u16,
    style_set: StyleSet,
}

/// Simply calls set_cell for every argument you give. \
/// use: `set_row(table, row, \[value0, value1, ...\])
#[macro_export]
macro_rules! set_table_row {
        ($table:expr, $row:expr, [$($value:expr),* $(,)?]) => {{
            let table = &mut $table;
            let row: usize = $row;
            let mut col = 0;
            $(
                table.set_cell(col, row, $value);
                col += 1;
            )*
        }};
    }

pub type StandardTable = Table<StandardColumnType>;
pub use builder::TableBuilder;

pub struct Row<'a, C: Cell> {
    cells: &'a mut [C],
}

pub struct RowMut<'a, C: Cell> {
    cells: &'a mut [C],
}

impl<'a, C: Cell> Row<'a, C> {
    pub fn cell(&self, index: usize) -> &C {
        &self.cells[index]
    }
}

impl<'a, C: Cell> RowMut<'a, C> {
    pub fn cell(&self, index: usize) -> &C {
        &self.cells[index]
    }

    pub fn cell_mut(&mut self, index: usize) -> &mut C {
        &mut self.cells[index]
    }

    pub fn set(&mut self, data: &[C]) {
        for (s, o) in self.cells.iter_mut().zip(data) {
            *s = o.clone();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    Fixed(u16),
    FixedRelative(f32),
    Growable(u16, u16), // min, max
    Adaptative {
        min: u16,
        max: u16,
        offset: u16, // offset from the longest cell in the column
    },
}

impl<T: ColumnType> Displayable for Table<T> {
    fn get_chars(&mut self) -> crate::CharContainer<'_> {
        let mut buffer = Vec::new();
        let mut char_container = crate::CharContainer::new_empty();

        let column_positions = self
            .computed_column_widths
            .iter()
            .scan(0, |acc, e| {
                let r = Some(*acc);
                *acc += *e;
                r
            })
            .collect::<Vec<_>>();

        // Titles
        let mut titles_buffer = Vec::new();
        char_container.set_format(self.style_set.titles);
        for (col_idx, col_title) in self.titles.iter().enumerate() {
            titles_buffer.extend(
                col_title
                    .chars()
                    .enumerate()
                    .map(|(i, e)| (i as u16 + column_positions[col_idx], 0, e)),
            );
            for x in col_title.len()..(self.computed_column_widths[col_idx] as _) {
                titles_buffer.push((x as u16 + column_positions[col_idx], 0, ' '));
            }
        }
        char_container.add_owned(titles_buffer);
        char_container.set_format(attributes::NORMAL | colors::back::GREY);

        let mut rows_buffer = Vec::new();
        for row_idx in 0..self.current_num_rows {
            let row = self.row(row_idx as usize);
            let fmt = if row_idx % 2 == 0 {
                self.style_set.even
            } else {
                self.style_set.odd
            };

            char_container.set_format(fmt);
            //TODO: all the formatting stuff
            for col_idx in 0..self.columns.len() {
                row[col_idx].get_chars(&mut buffer);
                rows_buffer.extend(
                    buffer
                        .iter()
                        .enumerate()
                        .map(|(i, e)| (i as u16 + column_positions[col_idx], row_idx + 1, *e)),
                );
                // Add whitespaces..
                for x in buffer.len()..(self.computed_column_widths[col_idx] as _) {
                    rows_buffer.push((x as u16 + column_positions[col_idx], row_idx + 1, ' '));
                }
                char_container.add_owned(rows_buffer.clone());
                rows_buffer.clear();
                buffer.clear();
            }
        }

        char_container
    }

    fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
        if let crate::Event::Resize(x, y) = event {
            self.compute_column_widths(x);
            self.max_rows = y - 1;
        }
    }
}

impl<T: ColumnType> Table<T> {
    fn new() -> Self {
        Self {
            titles: vec![],
            columns: vec![],
            column_widths: vec![],
            cells: vec![],
            computed_column_widths: vec![],
            current_num_rows: 0,
            max_rows: 0,
            style_set: StyleSet::new(),
        }
    }

    fn compute_column_widths(&mut self, available_width: u16) {
        let mut adaptative_sizes = vec![0; self.column_widths.len()];
        for (col_idx, &_width) in self
            .column_widths
            .iter()
            .enumerate()
            .filter(|(_i, e)| match e {
                ColumnWidth::Adaptative { .. } => true,
                _ => false,
            })
        {
            let max_len = (0..self.current_num_rows as usize)
                .map(|j| self.row(j as _)[col_idx].get_width())
                .max()
                .unwrap_or(0);
            let max_len = max_len.max(self.titles[col_idx].len() as u16);
            assert!(
                max_len > 0,
                "0 len detected. 0 means the cell will use as many chars as possible, and is incompatible to a column set to adaptative width. Please use a Growable column instead."
            );
            let ColumnWidth::Adaptative { min, max, offset } = self.column_widths[col_idx] else {
                unreachable!()
            };
            adaptative_sizes[col_idx] = max_len.clamp(min, max) + offset;
        }

        let mut growable_indexes = vec![];
        let total_fixed_width = self
            .column_widths
            .iter()
            .enumerate()
            .fold(0, |acc, (i, e)| {
                acc + match e {
                    ColumnWidth::Fixed(w) => *w,
                    ColumnWidth::FixedRelative(r) => (available_width as f32 * *r) as u16,
                    ColumnWidth::Growable(min, max) => {
                        growable_indexes.push(i);
                        *min
                    }
                    ColumnWidth::Adaptative { .. } => adaptative_sizes[i],
                }
            });

        let mut growable_sizes = vec![0; self.column_widths.len()];
        let mut remaining_space = available_width.saturating_sub(total_fixed_width);
        let mut growable_count = growable_indexes.len() as u16;

        let mut filled_a_column = false;
        while remaining_space > 0 && growable_count > 0 {
            let growable_added_width = remaining_space / growable_count;
            let mut extra_spaces = remaining_space % growable_count;

            for &i in &growable_indexes {
                let ColumnWidth::Growable(min, max) = self.column_widths[i] else {
                    unreachable!()
                };
                let current_width = min + growable_added_width;
                if current_width >= max {
                    growable_sizes[i] = max;
                    remaining_space = remaining_space.saturating_sub(max);
                    filled_a_column = true;
                    growable_count -= 1;
                    break;
                }
            }

            if !filled_a_column {
                for &i in &growable_indexes {
                    let ColumnWidth::Growable(min, max) = self.column_widths[i] else {
                        unreachable!()
                    };
                    let mut current_width = min + growable_added_width;
                    if extra_spaces > 0 && current_width < max {
                        current_width += 1;
                        extra_spaces -= 1;
                    }
                    growable_sizes[i] = current_width;
                }

                break;
            }
        }

        self.computed_column_widths.clear();
        for (i, width) in self.column_widths.iter().enumerate() {
            let computed_width = match width {
                ColumnWidth::Fixed(w) => *w,
                ColumnWidth::FixedRelative(r) => (available_width as f32 * *r) as u16,
                ColumnWidth::Growable(..) => growable_sizes[i],
                ColumnWidth::Adaptative { .. } => adaptative_sizes[i],
            };
            self.computed_column_widths.push(computed_width);
        }
    }

    fn add_row(&mut self) {
        let default_row = self.columns.iter().map(|c| c.default_cell());
        self.cells.extend(default_row);
    }

    pub fn row(&self, index: usize) -> &[T::CellType] {
        let idx = index * self.columns.len();
        if idx + self.columns.len() <= self.cells.len() {
            &self.cells[idx..idx + self.columns.len()]
        } else {
            panic!(
                "Row index out of bounds: {} (max {})",
                index,
                self.cells.len() / self.columns.len()
            );
        }
    }

    pub fn row_mut(&mut self, index: usize) -> RowMut<'_, T::CellType> {
        let idx = index * self.columns.len();
        if idx + self.columns.len() <= self.cells.len() {
            RowMut {
                cells: &mut self.cells[idx..idx + self.columns.len()],
            }
        } else {
            panic!(
                "Row index out of bounds: {} (max {})",
                index,
                self.cells.len() / self.columns.len()
            );
        }
    }

    pub fn column_type(&self, index: usize) -> T {
        self.columns[index].clone()
    }

    pub fn cell(&self, col: usize, row: usize) -> &T::CellType {
        &self.cells[col + row * self.columns.len()]
    }

    pub fn set_cell<V: IntoCell<T>>(&mut self, col: usize, row: usize, value: V) {
        let cell_ref = &mut self.cells[col + row * self.columns.len()];
        value.into_cell(&self.columns[col], cell_ref);
    }
}

mod builder {
    use super::*;

    pub struct New;
    pub struct TypeSelected;

    //TODO constraint T: ColumnType,
    pub struct TableBuilder<T: ColumnType, S> {
        table: Table<T>,
        _state: S,
    }

    impl TableBuilder<StandardColumnType, New> {
        pub fn new_standard() -> TableBuilder<StandardColumnType, TypeSelected> {
            TableBuilder {
                table: Table::new(),
                _state: TypeSelected,
            }
        }
    }

    impl<T: ColumnType> TableBuilder<T, New> {
        pub fn new_custom() -> TableBuilder<T, TypeSelected> {
            TableBuilder {
                table: Table::new(),
                _state: TypeSelected,
            }
        }
    }

    impl<T: ColumnType> TableBuilder<T, TypeSelected> {
        pub fn with_column(
            mut self,
            column_type: T,
            column_width: ColumnWidth,
            name: &str,
        ) -> Self {
            self.table.columns.push(column_type);
            self.table.column_widths.push(column_width);
            self.table.titles.push(name.to_string());
            self
        }

        pub fn with_columns(mut self, columns: Vec<(T, ColumnWidth, &str)>) -> Self {
            for (column_type, width, name) in columns {
                self = self.with_column(column_type, width, name);
            }
            self
        }

        pub fn with_style_set(mut self, style_set: StyleSet) -> Self {
            self.table.style_set = style_set;
            self
        }

        pub fn with_n_rows(mut self, n: u16) -> Self {
            self.table.max_rows = n;
            for _ in 0..n {
                self.table.add_row();
            }
            self.table.current_num_rows = n;
            self
        }

        pub fn build(self) -> Table<T> {
            self.table
        }
    }
}
