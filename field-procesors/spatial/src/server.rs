
use std::sync::Arc;
use std::sync::atomic::{AtomicBool,Ordering};


pub struct ServerConfiguration {
  pub world_id: i32,
  pub region_id: i32
}



pub fn start(config: &ServerConfiguration, exit_requested: Arc<AtomicBool>)  {

  while !exit_requested.load(Ordering::SeqCst) {
        
  }
}