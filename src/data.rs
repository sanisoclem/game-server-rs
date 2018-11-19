

use comms;
pub struct InputPacket {
    pub dummy: i16
}


pub struct OutputPacket{
    pub dummy: i32
}

impl comms::InputDatagram for InputPacket {
    fn decode(_: &[u8]) -> InputPacket {
        InputPacket {
            dummy: 0
        }
    }
}


impl comms::OutputDatagram for OutputPacket {
    fn encode(s: &OutputPacket, buf: &mut [u8]) {
        // do nothing for now
    }
}