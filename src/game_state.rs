

use std::net::SocketAddr;

pub trait WorldBounds {
    fn normalize_location(self: &Self, loc: &mut Location);
}

pub enum PlayerState {
    Idle,
}

pub struct Location {
    pub x: i32,
    pub y: i32,
}

pub struct PlayerInfo {
    pub uid: i32,
    pub location: Location,
    pub rot_z: f32,
    pub speed: f32,
    pub active: bool,
    pub addr: Option<SocketAddr>
}

pub struct SimpleWorldState {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32
}

impl WorldBounds for SimpleWorldState {
    fn normalize_location(self: &Self, loc: &mut Location) {
        if loc.x < self.min_x  {
            loc.x = self.min_x;
        } else if loc.x > self.max_x {
            loc.x = self.max_x;
        }
        if loc.y < self.min_y {
            loc.y = self.min_y;
        } else if loc.y > self.max_y {
            loc.y = self.max_y
        }
    }
}