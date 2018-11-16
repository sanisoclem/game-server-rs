mod comms;
mod server;
mod data;

fn main() {
    // TODO: get configure from somewhere?
    let config = server::ServerConfig {
        in_address: String::from("127.0.0.1:34254")
    };

    server::start(&config);
}
