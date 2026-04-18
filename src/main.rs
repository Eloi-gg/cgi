mod cgi;
struct CustomWidget {
    data: String,
}

impl cgi::Displayable for CustomWidget {
    fn display(&self) {
        println!("Displaying CustomWidget with data: {}", self.data);
    }

    fn name(&self) -> String {
        format!("CustomWidget {}", self.data)
    }
}


fn main() {
    let mut app = cgi::Application::new();

    let my_widget1 = CustomWidget {
        data: "Hello".to_string(),
    };
    let my_widget2 = CustomWidget {
        data: "World".to_string(),
    };
    let mut my_widget1 = cgi::Widget::new(my_widget1);
    let mut my_widget2 = cgi::Widget::new(my_widget2);

    let mut layout = cgi::LayoutBuilder::new();
    layout.add_widget(&my_widget1);
    layout.add_widget(&my_widget2);

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
