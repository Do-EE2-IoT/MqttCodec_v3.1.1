use bytes::{Buf, BufMut, BytesMut};

use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;
use crate::mqtt::utils;

#[derive(Debug, PartialEq)]
pub struct Last4BitsFixHeader {
    pub dup_flag: u8,
    pub qos_level: u8,
    pub retain: u8,
}

#[derive(Debug, PartialEq)]
pub struct Publish {
    pub last4bits_fix_header: Last4BitsFixHeader,
    pub topic_name: String,
    pub packet_id: Option<u16>,
    pub payload: String,
}

impl Encode for Publish {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError> {
        if self.last4bits_fix_header.qos_level > 2 {
            println!("Qos must be 0, 1 or 2");
            return Err(EncodeError::UnexpectedEndOfInput);
        }
        buffer.put_u8(
            (ControlPackets::Publish as u8)
                | self.last4bits_fix_header.dup_flag << 3
                | self.last4bits_fix_header.qos_level << 1
                | self.last4bits_fix_header.retain,
        );
        let remain_length = match self.last4bits_fix_header.qos_level {
            0 => 2 + self.topic_name.len() + self.payload.len(),
            _ => 2 + self.topic_name.len() + 2 + self.payload.len(),
        };
        buffer.put_u8(remain_length as u8);
        utils::encode_utf8(buffer, self.topic_name.as_str());
        if self.last4bits_fix_header.qos_level > 0 {
            if let Some(packet_id) = self.packet_id {
                buffer.put_u16(packet_id);
            }
        }
        buffer.put_slice(self.payload.as_bytes());
        Ok(())
    }
}

impl Decode for Publish {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError> {
        let first_byte = buffer.get_u8();
        let header = first_byte & 0xF0;
        println!("Get Publish Packet 0x{:02x}", header);
        let dup_flag = (first_byte >> 3) & 0x01;
        let qos_level = (first_byte >> 1) & 0x01;
        let retain = (first_byte) & 0x01;
        buffer.get_u8();
        if qos_level == 0 {
            let topic_name = utils::decode_utf8(buffer).unwrap();
            let payload: String =
                utils::decode_utf8_with_length(buffer, buffer.remaining()).unwrap();
            buffer.clear();
            Ok(Packet::Publish(Self {
                last4bits_fix_header: Last4BitsFixHeader {
                    dup_flag,
                    qos_level,
                    retain,
                },
                topic_name,
                packet_id: None,
                payload,
            }))
        } else if qos_level == 1 || qos_level == 2 {
            let topic_name = utils::decode_utf8(buffer).unwrap();
            let packet_id = buffer.get_u16();
            let payload = utils::decode_utf8_with_length(buffer, buffer.remaining()).unwrap();
            Ok(Packet::Publish(Self {
                last4bits_fix_header: Last4BitsFixHeader {
                    dup_flag,
                    qos_level,
                    retain,
                },
                topic_name,
                packet_id: Some(packet_id),
                payload,
            }))
        } else {
            Err(DecodeError::UnexpectedEndOfInput)
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Publish;

    #[test]
    fn test_publish_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let publish_packet = Publish {
            last4bits_fix_header: super::Last4BitsFixHeader {
                dup_flag: 1,
                qos_level: 0,
                retain: 1,
            },
            topic_name: "/hello".to_string(),
            packet_id: None,
            payload: "Hanoi university of science and technology".to_string(),
        };

        publish_packet.encode(&mut buffer).unwrap();

        let packet = Publish::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Publish(decoded_publish_packet) => {
                assert_eq!(decoded_publish_packet, publish_packet)
            }
            _ => panic!("Expected a ConnackPacket but got something else"),
        }
    }

    #[test]
    fn test_qos_invalid_with_encode() {
        let mut buffer = BytesMut::new();
        let publish_packet = Publish {
            last4bits_fix_header: super::Last4BitsFixHeader {
                dup_flag: 1,
                qos_level: 100,
                retain: 1,
            },
            topic_name: "/hello".to_string(),
            packet_id: None,
            payload: "Hanoi university of science and technology".to_string(),
        };

        assert!(publish_packet.encode(&mut buffer).is_err());
    }
}
