use comms;
use std::net::SocketAddr;
use data_proto::{InputPacket,OutputPacket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct ServerConfig {
    pub in_address: String
}

pub fn start(config: &ServerConfig, exit_requested: Arc<AtomicBool>) {
    // -- start comms
    let mut comms_mgr : comms::CommsManager<InputPacket,OutputPacket> = comms::start_udp(&config.in_address);

    let mut input_buf: Box<Vec<(SocketAddr,InputPacket)>> = Box::new(Vec::new());
    let mut output_buf: Box<Vec<(SocketAddr,OutputPacket)>> = Box::new(Vec::new());;

    // -- main loop
    while !exit_requested.load(Ordering::SeqCst) {
        // swap input buffers
        input_buf =  comms_mgr.swap_inputs(input_buf);

        // calculate changes to game state
        while let Some((s,item)) = input_buf.pop() {
            //println!("received message: {0}", item);
            output_buf.push((s,OutputPacket{
                user: item.user,
                state: item.action,
                loc_x: item.loc_x,
                loc_y: item.loc_y,
            }));
        }

        //std::thread::sleep_ms(5000);

        // -- calculate contents of output buffer

        // -- swap buffers
        output_buf = comms_mgr.swap_outputs(output_buf);
        output_buf.clear();
    }
    println!("cleaning up");

    // -- clenaup threads
    comms_mgr.finalize();

    println!("Exiting");
}