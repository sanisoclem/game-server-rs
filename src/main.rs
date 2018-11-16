mod comms;
mod input_udp;
mod server;

fn main() {
    // TODO: get configure from somewhere?
    let config = server::ServerConfig {
        in_address: String::from("127.0.0.1:34254")
    };

    server::start(&config);
}
