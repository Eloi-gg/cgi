use std::cell::RefCell;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Mutex;

pub(crate) const CONNECTION_NAME: &str = "CGI log";
pub(crate) const CONNECTION_IP: &str = "127.0.0.1";
pub(crate) const CONNECTION_PORT: u16 = 4000;

pub fn get_dbg_window_exe_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/debug/dbg_window")
}

static LOGGER: Mutex<Option<crate::debug::dbg_window::connect::DebugConsole>> =
    Mutex::new(None);

pub(crate) fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let stream = crate::debug::dbg_window::connect::connect_to_server(
        CONNECTION_NAME,
        CONNECTION_IP,
        CONNECTION_PORT,
        Some(std::time::Duration::from_secs(5)),
    )?;
    *LOGGER.lock().unwrap() = Some(stream);
    Ok(())
}

pub(crate) fn log(msg: &str) {
    if let Some(ref mut stream) = *LOGGER.lock().unwrap() {
        stream.send_message(msg);
    } 
}
