#[macro_use]
extern crate ctrlc;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod server;

pub const GAME_FIELD_ID:i32=1;


fn main() {
    // CtrlC support
    let exit_requested = Arc::new(AtomicBool::new(false));
    let er = exit_requested.clone();
    ctrlc::set_handler(move || {
        er.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let config = server::ServerConfiguration {
        world_id: 1,
        region_id: 1
    };
    
    server::start(&config, exit_requested)
}