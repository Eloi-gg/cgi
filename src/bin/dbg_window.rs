use std::env;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

const PROGRAM_NAME: &str = "dbg_window";

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
        println!("DbgServer received: {}", line.unwrap());
    }
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Client disconnected, press any key to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn spawn_term(app: &str, addr: &str) {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "cmd", "/K", &format!("{} {}", app, addr)])
            .spawn()
            .expect("Failed to launch DbgServer");
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(&["-a", "Terminal", app, addr])
            .spawn()
            .expect("Failed to launch DbgServer");
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("gnome-terminal")
            .args(&["--", app, addr])
            .spawn()
            .expect("Failed to launch DbgServer");
    }
}

pub mod create {
    pub fn spawn_server(dbg_window_path: &str, addr: &str, port: u16) {
        super::spawn_term(dbg_window_path, &format!("{}:{}", addr, port));
    }
}

pub mod connect {
    use std::{
        io::{BufRead as _, BufReader, Write},
        net::{TcpListener, TcpStream}, time,
    };

    pub struct DebugConsole {
        name: String,
        stream: TcpStream,
    }

    pub fn connect_to_server(
        name: &str,
        addr: &str,
        port: u16,
        timeout: Option<time::Duration>,
    ) -> Result<DebugConsole, std::io::Error> {
        if let Some(timeout) = timeout {
            let start_time = time::Instant::now();
            while start_time.elapsed() < timeout {
                match TcpStream::connect(format!("{}:{}", addr, port)) {
                    Ok(stream) => {
                        return Ok(DebugConsole {
                            name: name.to_string(),
                            stream,
                        });
                    }
                    Err(_) => {
                        std::thread::sleep(time::Duration::from_millis(100));
                    }
                }
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Failed to connect to server within timeout",
            ));
        } else {
            let stream = TcpStream::connect(format!("{}:{}", addr, port))?;
            Ok(DebugConsole {
                name: name.to_string(),
                stream,
            })
        }
    }

    impl DebugConsole {
        pub fn send_message(&mut self, msg: &str) {
            self.stream.write_all(msg.as_bytes()).unwrap();
            self.stream.write_all(b"\n").unwrap();
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }

    impl std::fmt::Display for DebugConsole {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "DebugConsole {}@{}:{}",
                self.name,
                self.stream.local_addr().unwrap().ip(),
                self.stream.peer_addr().unwrap().port()
            )
        }
    }
}
