use std::env;
use std::io::{BufRead, BufReader};

fn main() {
    let args: Vec<String> = env::args().collect();
    let addr = args.get(1).expect("Usage: app2_display <server_addr:port>");

    println!("DbgServer: Starting and listening on {}", addr);
    let listener = std::net::TcpListener::bind(addr).expect("Could not bind to address");
    // Accept a single connection
    let stream = listener.incoming().next().unwrap().unwrap();
    let reader = BufReader::new(stream);

    println!("DbgServer: Connected to {}", addr);

    for line in reader.lines() {
        println!("{}", line.unwrap());
    }
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Client disconnected, press any key to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}