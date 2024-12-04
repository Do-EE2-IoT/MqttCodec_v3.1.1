use bytes::{Buf, BufMut};

use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;
use crate::mqtt::utils;

#[derive(Debug, PartialEq)]
pub struct Unsubscribe {
    pub packet_id: u16,
    pub topic: String,
}

impl Encode for Unsubscribe {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Unsubscribe as u8);
        let remaining_length = 2 + 2 + self.topic.len();
        buffer.put_u8(remaining_length as u8);
        buffer.put_u16(self.packet_id);
        utils::encode_utf8(buffer, self.topic.as_str());
        Ok(())
    }
}

impl Decode for Unsubscribe {
    fn decode(buffer: &mut bytes::BytesMut) -> Result<Packet, DecodeError> {
        let header = buffer.get_u8();
        println!("Get Unsubscribe packet 0x:{:02x}", header);
        buffer.get_u8();
        let packet_id = buffer.get_u16();
        let topic = utils::decode_utf8(buffer).expect("Must be decode utf8 success");
        buffer.clear();
        Ok(Packet::Unsubscribe(Self { packet_id, topic }))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Unsubscribe;

    #[test]
    fn test_unsub_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let unsub_packet = Unsubscribe {
            packet_id: 555,
            topic: "Hello".to_string(),
        };
        unsub_packet.encode(&mut buffer).unwrap();
        let packet = Unsubscribe::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Unsubscribe(decoded_unsubscribe) => {
                assert_eq!(decoded_unsubscribe, unsub_packet)
            }
            _ => panic!("Expected a Unsub packet but got something else"),
        }
    }
}
