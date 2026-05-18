use cgi::Displayable;
use cgi::widget::WidgetBuilder;
use cgi::*;

struct CustomWidget {
    data: String,
}

impl Displayable for CustomWidget {
    fn display(&self) {
        println!("Displaying CustomWidget with data: {}", self.data);
    }

    fn name(&self) -> String {
        format!("CustomWidget {}", self.data)
    }

    fn get_changed_chars(&self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>) {
        todo!()
    }

    fn on_event(&mut self, event: cgi::Event) {
        todo!()
    }
}

fn scenario_1() {
    use cgi::factory_widgets::{Listener, text::*};

    let mut app = cgi::Application::new();
    let text1 = " Hello, World! \n
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. \n
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. \n
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. \n
        ";
    let text2 = "Le tonnerre est un son produit par l'expansion brutale de la fine colonne d'air qui a été chauffée très rapidement
        par la foudre au cours d'un orage[1]. Il se manifeste sous la forme d'un claquement sec ou d'un roulement sourd dont l'intensité
        est d'autant plus forte que le phénomène de foudre qui le provoque est plus proche du lieu où se situe l'observateur,
        à décharge électrostatique équivalente, sans vent ni relief et à moins de conditions de propagation anormale du son dans l'air[2].";

    let text_widget = WidgetBuilder::new(TextBox::new(text1, Listener::empty()))
        .with_outline(cgi::symbols::OutlineStyle::Rounded)
        .build();
    let text_widget2 = WidgetBuilder::new(TextBox::new(text2, Listener::empty()))
        .with_outline(cgi::symbols::OutlineStyle::Rounded)
        .build();

    let placement = WidgetPlacement::new(0, 0, 24, 8);
    let placement2 = placement.shift(30, 4).expand_or_shrink(-3, 0).with_width(0.5);

    let layout = Layout::new()
        .with_widget(&text_widget, placement)
        .with_widget(&text_widget2, placement2);
    app.set_layout_behaviour(|(w, h)| "MainLayout".to_string());
    app.add_layout("MainLayout", layout);

    app.run();
}

fn main() {
    scenario_1();
    return;

    let mut app = cgi::Application::new();

    let my_widget1 = CustomWidget {
        data: "Hello".to_string(),
    };
    let my_widget2 = CustomWidget {
        data: "World".to_string(),
    };
    let mut my_widget1 = cgi::Widget::new(my_widget1);
    let mut my_widget2 = cgi::Widget::new(my_widget2);

    let mut layout = cgi::Layout::new();
    let mut placements = [cgi::layout::WidgetPlacement::default(); 2];
    cgi::layout::WidgetPlacement::fullscreen().split(2, 1, &mut placements);

    layout.add_widget(&my_widget1, placements[0]);
    layout.add_widget(&my_widget2, placements[1]);

    app.add_layout("MainLayout", layout);
    app.set_layout_behaviour(|(w, h)| {
        println!("b: size {w}, {h}");
        "MainLayout".to_string()
    });

    std::thread::spawn(move || {
        for i in 0..5 {
            {
                let mut lock = my_widget1.edit();
                lock.data = format!("Changed {}", i);
            }
            my_widget1.repaint();
            std::thread::sleep(std::time::Duration::from_secs_f32(1.0));
        }
    });

    app.run();
}
