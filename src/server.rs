use comms;
use data::{InputPacket,OutputPacket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct ServerConfig {
    pub in_address: String
}

pub fn start(config: &ServerConfig, exit_requested: Arc<AtomicBool>) {
    // -- start comms
    let mut commsMgr : comms::CommsManager<InputPacket,OutputPacket> = comms::start_udp(&config.in_address);

    let mut inputBuffer: Box<Vec<InputPacket>> = Box::new(Vec::new());
    let mut outputBuffer: Box<Vec<OutputPacket>> = Box::new(Vec::new());;

    // -- main loop
    while !exit_requested.load(Ordering::SeqCst) {
        // swap input buffers
        inputBuffer =  commsMgr.swap_inputs(inputBuffer);

        // calculate changes to game state
        while let Some(item) = inputBuffer.pop() {
            println!("received message: {0}", item.dummy);
        }

        std::thread::sleep_ms(5000);

        // -- calculate contents of output buffer

        // -- swap buffers
        //outputBuffer = commsMgr.swap_outputs(outputBuffer);
    }
    println!("cleaning up");

    // -- clenaup threads
    commsMgr.finalize();

    println!("Exiting");
}