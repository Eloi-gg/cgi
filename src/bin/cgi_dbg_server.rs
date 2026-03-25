// This program is a simple example of how to use the dbg_window crate to create a debug server and connect to it from a Rust application. It can be deleted

mod dbg_window;

fn main() {
    let dbg_window_path = std::env::current_dir().unwrap().join("target/debug/dbg_window");
    dbg_window::create::spawn_server(dbg_window_path.to_str().unwrap(), "127.0.0.1", 4000);
    let mut console = dbg_window::connect::connect_to_server("MainApp", "127.0.0.1", 4000, Some(std::time::Duration::from_secs(5)))
        .expect("Failed to connect to DbgServer");

    for i in 0..5 {
        console.send_message(&format!("Hello from MainApp! {}", i));
        println!("Sent message {} to DbgServer", i);
        std::thread::sleep(std::time::Duration::from_secs_f32(1.0));
    }
}
