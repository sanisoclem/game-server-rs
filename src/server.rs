use comms;
use std::net::SocketAddr;
use data_proto::{InputPacket,OutputPacket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use game_state::{Location,PlayerInfo,SimpleWorldState,WorldBounds};
use std::time::{Duration, Instant};

pub struct ServerConfig {
    pub in_address: String
}

pub fn start(config: &ServerConfig, exit_requested: Arc<AtomicBool>) {
    // -- start comms
    let mut comms_mgr : comms::CommsManager<InputPacket,OutputPacket> = comms::start_udp(&config.in_address);

    let mut input_buf: Box<Vec<(SocketAddr,InputPacket)>> = Box::new(Vec::new());
    let mut output_buf: Box<Vec<(SocketAddr,OutputPacket)>> = Box::new(Vec::new());;

    let world_state = SimpleWorldState {
        min_x: 0,
        min_y: 0,
        max_x: 1000000,
        max_y: 1000000
    };
    let mut player_state: Vec<PlayerInfo> = Vec::with_capacity(1024);

    // -- TODO: allow player join
    // -- hard code 1 player
    player_state.push(PlayerInfo {
        uid: 1,
        location: Location {
            x: 100,
            y: 100
        },
        rot_z: 0.0,
        speed: 0.0,
        active: true,
        addr: None
    });

    let mut this_tick;
    let mut last_tick = Instant::now();

    // -- main loop
    while !exit_requested.load(Ordering::SeqCst) {
        this_tick = Instant::now();
        let tick_delta =  (this_tick - last_tick).subsec_millis() as f32 / 1000.0;

        // swap input buffers
        input_buf =  comms_mgr.swap_inputs(input_buf);

        output_buf.clear();

        // process inputs
        while let Some((s,item)) = input_buf.pop() {
            // -- check if we have a player entry
            if player_state.len() > item.user as usize {
                let user = &mut player_state[item.user as usize];

                // -- skip if inactive or uid does not match
                if !user.active || user.uid != item.uid {
                    continue;
                }

                user.addr = Some(s);

                // can player set speed at any time? is there acceleration?
                user.speed = item.speed;
                // -- is there rot speed?
                user.rot_z = item.rot_z;
            } else {
                // -- unknown player, drop it like its hot
            }
        }

        // -- update game state
        for ctr in 0..player_state.len()-1 {
            let mut user = &mut player_state[ctr];

            // -- update player position
            user.location.x += (user.rot_z.cos() * user.speed * tick_delta).floor() as i32;
            user.location.y += (user.rot_z.sin() * user.speed  * tick_delta).floor() as i32;

            // -- ensure position is correct
            world_state.normalize_location(&mut user.location);
        }

        // -- todo: nanosleep if we finished early

        // -- calculate contents of output buffer

        // -- swap buffers
        output_buf = comms_mgr.swap_outputs(output_buf);

        last_tick = this_tick;
    }
    println!("cleaning up");

    // -- clenaup threads
    comms_mgr.finalize();

    println!("Exiting");
}