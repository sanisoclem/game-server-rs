#[macro_use]
extern crate prost_derive;
extern crate ctrlc;
extern crate mio;
extern crate mio_extras;
extern crate prost;
extern crate bytes;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Include the `items` module, which is generated from items.proto.
pub mod data_proto {
    include!(concat!(env!("OUT_DIR"), "/data.proto.rs"));
}
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
