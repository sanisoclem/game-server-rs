extern crate ctrlc;
extern crate mio;
extern crate mio_extras;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod comms;
mod server;
mod data;

fn main() {

    let exit_requested = Arc::new(AtomicBool::new(false));
    let er = exit_requested.clone();
    ctrlc::set_handler(move || {
        er.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // TODO: get configure from somewhere?
    let config = server::ServerConfig {
        in_address: String::from("127.0.0.1:34254")
    };

    server::start(&config, exit_requested);
}
