
use comms;

pub struct ServerConfig {
    pub in_address: String
}

pub fn start(config: &ServerConfig) {
    // -- start the input config
    let commsMgr = comms::start_udp(&config.in_address);
}