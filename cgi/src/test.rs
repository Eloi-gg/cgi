//! Common test utilities and structures used across all test modules
//!
//! This module provides reusable test helpers and widget implementations for testing.

#[cfg(test)]
mod inner {
    pub(crate) use crate::rendering::TestOutput;

    use crate::{Displayable, Event, widget::Widget};

    pub(crate) static TESTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/");

    /// A simple widget that fills its space with a single character.
    /// Useful for testing rendering and layout.
    pub struct FillWidget {
        pub ch: char,
        buffer: Vec<(u16, u16, char)>,
    }

    impl Displayable for FillWidget {
        fn display(&self) {
            // No-op for testing
        }

        fn name(&self) -> String {
            format!("FillWidget '{}'", self.ch)
        }

        fn get_changed_chars(&mut self, size: (u16, u16)) -> &[(u16, u16, char)] {
            self.buffer.clear();
            for y in 0..size.1 {
                for x in 0..size.0 {
                    self.buffer.push((x, y, self.ch));
                }
            }
            &self.buffer
        }

        fn on_event(&mut self, event: Event, actions: &mut crate::ActionList) {
            // No-op for testing
        }
    }

    impl FillWidget {
        /// Creates a new FillWidget with the given character
        pub fn new(ch: char) -> Self {
            Self { ch, buffer: Vec::new() }
        }
    }

    /// Generator for creating FillWidget instances with incrementing character codes
    pub struct FillGenerator {
        count: u32,
    }

    impl FillGenerator {
        /// Creates a new FillGenerator starting from '0'
        pub fn new() -> Self {
            Self { count: 0 }
        }

        /// Gets the next FillWidget
        pub fn next(&mut self) -> FillWidget {
            let dummy = FillWidget::new(self.count.to_string().chars().next().unwrap());
            self.count += 1;
            dummy
        }

        /// Creates n FillWidget instances wrapped in Widget
        pub fn get_n_widgets(&mut self, n: u32) -> Vec<Widget<FillWidget>> {
            (0..n).map(|_| Widget::new(self.next())).collect()
        }
    }

    impl Default for FillGenerator {
        fn default() -> Self {
            Self::new()
        }
    }

    /// A simple dummy widget for testing that holds a numeric value
    pub struct Dummy {
        pub data: u32,
    }

    impl Displayable for Dummy {
        fn display(&self) {
            println!("Displaying Dummy with data: {}", self.data);
        }

        fn name(&self) -> String {
            format!("Dummy {}", self.data)
        }

        fn get_changed_chars(&mut self, size: (u16, u16)) -> &[(u16, u16, char)] {
            // No-op for testing
            return &[];
        }

        fn on_event(&mut self, _event: Event, actions: &mut crate::ActionList) {
            // No-op for testing
        }
    }

    impl Dummy {
        /// Creates a new Dummy widget with the given data value
        pub fn new(data: u32) -> Self {
            Self { data }
        }
    }

    /// Generator for creating Dummy instances with incrementing data values
    pub struct DummyGenerator {
        count: u32,
    }

    impl DummyGenerator {
        /// Creates a new DummyGenerator starting from 0
        pub fn new() -> Self {
            Self { count: 0 }
        }

        /// Gets the next Dummy widget
        pub fn next(&mut self) -> Dummy {
            let dummy = Dummy::new(self.count);
            self.count += 1;
            dummy
        }

        /// Creates n Dummy instances wrapped in Widget
        pub fn get_n_widgets(&mut self, n: u32) -> Vec<Widget<Dummy>> {
            (0..n).map(|_| Widget::new(self.next())).collect()
        }
    }

    impl Default for DummyGenerator {
        fn default() -> Self {
            Self::new()
        }
    }

    pub fn get_single_widget_rendered_text(
        widget: &Widget<impl Displayable + 'static>,
        size: (i32, i32),
    ) -> String {
        use crate::*;
        let mut output = crate::rendering::TestOutput::<100, 100>::new();
        output.change_size((size.0 as usize, size.1 as usize));
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(widget, placement);

        let layout = layout.render(size.0, size.1);
        layout.render_to_output(&mut output);

        output.to_string()
    }

    /// Asserts that rendered text matches the content of a test file
    ///
    /// # Arguments
    /// * `text` - The rendered text to compare
    /// * `file_name` - The name of the test file (relative to the tests directory)
    ///                 If the name doesn't end with ".txt", ".txt" will be appended
    pub fn assert_match_with_test_file(text: &str, file_name: &str) {
        let tests_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/");
        let extension = if file_name.ends_with(".txt") {
            ""
        } else {
            ".txt"
        };
        let expected_result =
            std::fs::File::open(format!("{}/{}{}", tests_dir, file_name, extension)).expect(
                &format!(
                    "Failed to open test file: {}{}{}",
                    tests_dir, file_name, extension
                ),
            );
        let expected_result =
            std::io::read_to_string(expected_result).expect("Failed to read test file content");

        let mut diff = 0;
        for i in 0..text.len().min(expected_result.len()) {
            if text.chars().nth(i) != expected_result.chars().nth(i) {
                diff = i;
            }
        }
        assert_eq!(
            text,
            expected_result,
            "Diff at index: {} ({}) but expected ({}) out of length {} vs {}",
            diff,
            text.chars().nth(diff).unwrap_or('?'),
            expected_result.chars().nth(diff).unwrap_or('?'),
            text.len(),
            expected_result.len()
        );
    }

    pub mod strings {

        pub fn lorem_ipsum_long() -> String {
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit\n"
                .repeat(15)
        }

        pub fn lorem_ipsum_short() -> &'static str {
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit\n"
        }
    }
}

// Re-export test utilities for use in test modules
#[cfg(test)]
pub use inner::*;
