#[macro_use]
extern crate prost_derive;
extern crate ctrlc;
extern crate mio;
extern crate mio_extras;
extern crate prost;
extern crate bytes;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use prost::Message;

// Include the `items` module, which is generated from items.proto.
pub mod data_proto {
    include!(concat!(env!("OUT_DIR"), "/data.proto.rs"));
}
mod comms;
mod server;

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

    thread::spawn(move || {
        thread::sleep_ms(5000);
        let socket = std::net::UdpSocket::bind("127.0.0.1:34255").unwrap();
        socket.connect("127.0.0.1:34254").unwrap();
        let mut buf = bytes::BytesMut::with_capacity(512);
        let mut buf2 = [0;512];
        let msg = data_proto::InputPacket {
            user: 1,
            action: 5,
            loc_x: 6,
            loc_y: 7
        };
        msg.encode(&mut buf).unwrap();
        println!("{:x?}", buf);

        loop {
            socket.send(&buf).unwrap();
            socket.recv(&mut buf2).unwrap();
        }
    });

    server::start(&config, exit_requested);
}
