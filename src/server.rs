use comms;
use data::{InputPacket,OutputPacket};

pub struct ServerConfig {
    pub in_address: String
}

pub fn start(config: &ServerConfig) {
    // -- start comms
    let mut commsMgr : comms::CommsManager<Vec<InputPacket>,Vec<OutputPacket>> = comms::start_udp(&config.in_address);

    let mut inputBuffer: Vec<InputPacket> = Vec::new();
    let mut outputBuffer: Vec<OutputPacket> = Vec::new();

    // -- main loop
    while config.in_address.len() == 0 /* dummy condition */ {
        // swap input buffers
        inputBuffer =  commsMgr.swap_inputs(inputBuffer);

        // calculate changes to game state

        // -- calculate contents of output buffer

        // -- swap buffers
        outputBuffer = commsMgr.swap_outputs(outputBuffer);
    }

    // -- clenaup threads
    commsMgr.finalize();
}