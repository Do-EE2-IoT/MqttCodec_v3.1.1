use bytes::{Buf, BufMut};

use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;
use crate::mqtt::utils;

#[derive(Debug, PartialEq)]
pub struct Subscribe {
    pub packet_id: u16,
    pub topic: String,
    pub qos: u8,
}

impl Encode for Subscribe {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        if self.qos > 2 {
            return Err(EncodeError::UnexpectedEndOfInput);
        }
        buffer.put_u8(ControlPackets::Subscribe as u8);
        let remaning_length = 2 + 2 + self.topic.len() + 1;
        buffer.put_u8(remaning_length as u8);
        buffer.put_u16(self.packet_id);
        utils::encode_utf8(buffer, self.topic.as_str());
        buffer.put_u8(self.qos);
        Ok(())
    }
}

impl Decode for Subscribe {
    fn decode(buffer: &mut bytes::BytesMut) -> Result<Packet, DecodeError> {
        let header = buffer.get_u8();
        buffer.get_u8();
        let packet_id = buffer.get_u16();
        println!(
            "Get Subscibe Packet {:02x} with packet id {:04x} ",
            header, packet_id
        );
        let topic = utils::decode_utf8(buffer).expect("Must be decode utf8 success");
        let qos = buffer.get_u8();
        if qos > 2 {
            return Err(DecodeError::UnexpectedEndOfInput);
        }
        println!("Client register topic : {} and qos {}", topic, qos);
        Ok(Packet::Subscribe(Self {
            packet_id,
            topic,
            qos,
        }))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Subscribe;

    #[test]
    fn test_subscribe_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let subscribe_packet = Subscribe {
            packet_id: 2344,
            topic: "/hello/123".to_string(),
            qos: 2,
        };
        subscribe_packet.encode(&mut buffer).unwrap();

        let packet = Subscribe::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Subscribe(decoded_subscribe) => {
                assert_eq!(decoded_subscribe, subscribe_packet);
            }
            _ => panic!("Expected a Pubrel packet but got something else"),
        }
    }

    #[test]
    fn test_invalid_qos() {
        let mut buffer = BytesMut::new();
        let subscribe_packet = Subscribe {
            packet_id: 2344,
            topic: "/hello/123".to_string(),
            qos: 3,
        };
        assert!(subscribe_packet.encode(&mut buffer).is_err());
    }
}
