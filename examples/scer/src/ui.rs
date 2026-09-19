use cgi::factory_widgets as fw;
use cgi::widget::WidgetBuilder;
use cgi::{Layout, Widget, WidgetPlacement};
use rand::Rng;

const SCREEN_SIZE: (i32, i32) = (16, 4);

const SMALL_LAYOUT: u8 = 0;

pub fn get_layout_id(size: (u16, u16)) -> u8 {
    SMALL_LAYOUT
}

pub fn small_layout() -> Layout {
    let screen = WidgetBuilder::new(fw::utils::Empty)
        .with_outline(cgi::symbols::OutlineStyle::Double)
        .build();
    let mut layout = Layout::new();
    layout.add_widget(
        &screen,
        WidgetPlacement::new_with_size(0, 0, SCREEN_SIZE.0, SCREEN_SIZE.1),
    );
    layout.append(modules::next_instruction(0, SCREEN_SIZE.1 + 1));
    layout.append(modules::registers(
        0,
        SCREEN_SIZE.1 + modules::NEXT_INSTRUCTION_SIZE.1 + 2,
    ));
    layout.append(modules::message_boards(
        0,
        SCREEN_SIZE.1 + modules::NEXT_INSTRUCTION_SIZE.1 + modules::REGISTERS_SIZE.1 + 3,
    ));
    layout
}

mod modules {
    use super::*;
    use cgi::factory_widgets::text::Wrapping;

    pub(super) const NEXT_INSTRUCTION_SIZE: (i32, i32) = (38, 4);
    pub(super) const REGISTERS_SIZE: (i32, i32) = (38, 6);

    pub struct MessageBoard(pub(super) fw::text::TextBox);

    impl MessageBoard {
        fn new() -> Self {
            let mut text_box =
                fw::text::TextBox::new("", fw::Listener::empty(), fw::text::TextAlign::Left);
            text_box.set_wrapping_mode(fw::text::Wrapping::Off);
            Self(text_box)
        }

        pub(super) fn add_message(&mut self, message: &str) {
            let old_text = self.0.text();
            let nex_text = message.to_owned() + "\n" + &old_text;
            self.0.set_text(&nex_text);
        }
    }

    pub(super) fn next_instruction(pos_x: i32, pos_y: i32) -> Layout {
        let empty_listener = fw::Listener::empty();

        let placement = WidgetPlacement::new_with_size(
            pos_x,
            pos_y,
            NEXT_INSTRUCTION_SIZE.0,
            NEXT_INSTRUCTION_SIZE.1,
        );
        let widget = &WidgetBuilder::new(
            fw::text::TextBox::new(
                "0b00000000100010000000000000100001\npush 0x21",
                empty_listener.clone(),
                fw::text::TextAlign::Left,
            )
            .with_wrapping_mode(Wrapping::Off),
        )
        .with_outline(cgi::symbols::OutlineStyle::Normal)
        .with_title("Next instruction")
        .build();

        Layout::new().with_widget(widget, placement)
    }

    pub(super) fn registers(pos_x: i32, pos_y: i32) -> Layout {
        let empty_listener = fw::Listener::empty();

        let register_title_widget_generator = |s: &str| {
            let mut tb =
                fw::text::TextBox::new(s, empty_listener.clone(), fw::text::TextAlign::Left);
            tb.set_style(
                cgi::text_formatting::attributes::BOLD | cgi::text_formatting::colors::GREY,
            );
            Widget::new(tb)
        };
        let register_value_widget_generator = || {
            let rand = rand::thread_rng().gen_range(0..0xFFFF);
            let tb = fw::text::TextBox::new(
                &format!("0x{:04X}", rand),
                empty_listener.clone(),
                fw::text::TextAlign::Left,
            );

            Widget::new(tb)
        };

        let mut registers_split = [WidgetPlacement::default(); 12];
        let registers_space = WidgetPlacement::new_with_size(pos_x, pos_y, 38, 6);
        registers_space.expand_or_shrink(-1, -1).shift(1, 0).split(
            3,
            4,
            true,
            &mut registers_split,
        );
        let register_titles_placements = registers_split.iter().map(|p| p.with_width(4));
        let register_values_placements = registers_split.iter().map(|p| p.shift_top_left(4, 0));

        let mut layout = Layout::new();
        for (i, p) in register_titles_placements.enumerate() {
            let register_title_widget = register_title_widget_generator(&format!("R{}: ", i % 10));
            layout.add_widget(&register_title_widget, p);
        }
        for (i, p) in register_values_placements.enumerate() {
            let register_value_widget = register_value_widget_generator();
            layout.add_widget(&register_value_widget, p);
        }
        layout.add_widget(
            &WidgetBuilder::new(fw::utils::Empty)
                .with_outline(cgi::symbols::OutlineStyle::Normal)
                .with_title("Registers")
                .build(),
            registers_space,
        );

        layout
    }

    pub(super) fn message_boards(pos_x: i32, pos_y: i32) -> Layout {
        use cgi::text_formatting::*;

        let mut layout = Layout::new();
        let mut old_messages = MessageBoard::new();
        let mut new_messages = MessageBoard::new();

        old_messages
            .0
            .set_style(attributes::DIM | attributes::ITALIC);
        new_messages.0.set_style(attributes::ITALIC);

        let new_placement = WidgetPlacement::new_with_size(pos_x, pos_y, 38, 6);
        let old_placement = new_placement.get_below().with_bottom_right_y(1.0);

        for i in 0..3 {
            new_messages.add_message(&format!("NEW {}", i));
        }
        for i in 0..14 {
            old_messages.add_message(&format!("OLD {}", i));
        }

        let old_messages_widget = WidgetBuilder::new(old_messages.0)
            .with_outline(cgi::symbols::OutlineStyle::Rounded)
            .with_title("Old Messages")
            .build();
        let new_messages_widget = WidgetBuilder::new(new_messages.0)
            .with_outline(cgi::symbols::OutlineStyle::Rounded)
            .with_title("New Messages")
            .build();
        layout.connect_and_add_widgets(
            &mut vec![old_messages_widget, new_messages_widget],
            &mut [old_placement, new_placement],
        );
        layout
    }
}
