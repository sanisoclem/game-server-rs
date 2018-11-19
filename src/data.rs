
use bytes;
use comms;
use data_proto::{InputPacket,OutputPacket};
use prost::Message;

impl comms::InputDatagram for InputPacket {
    fn deserialize(b: &[u8]) -> InputPacket {
        let buf = bytes::Bytes::from(b);
        InputPacket::decode(buf).unwrap()
    }
}


impl comms::OutputDatagram for OutputPacket {
    fn encode(self: &Self, buf: &mut [u8]) {
        //let mut bs = bytes::BytesMut::from(buf);
        //self.encode(bs);
    }
}


// pub fn create_large_shirt(color: String) -> items::Shirt {
//     let mut shirt = items::Shirt::default();
//     shirt.color = color;
//     shirt.set_size(items::shirt::Size::Large);
//     shirt
// }