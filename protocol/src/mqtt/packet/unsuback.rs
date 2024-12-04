use bytes::{Buf, BufMut, BytesMut};

use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;

#[derive(Debug, PartialEq)]
pub struct Unsuback {
    packet_id: u16,
}

impl Encode for Unsuback {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Unsuback as u8);
        buffer.put_u8(0x02);
        buffer.put_u16(self.packet_id);
        Ok(())
    }
}

impl Decode for Unsuback {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError> {
        if buffer.len() != 4 {
            Err(DecodeError::InvalidMessageFormat)
        } else {
            let header = buffer.get_u8();
            println!("Get Unsuback packet : {:02x}", header);
            buffer.get_u8();
            let packet_id = buffer.get_u16();
            buffer.clear();
            let unsuback_packet = Self { packet_id };
            Ok(Packet::Unsuback(unsuback_packet))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Unsuback;

    #[test]
    fn test_unsuback_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let unsuback = Unsuback { packet_id: 12344 };
        unsuback.encode(&mut buffer).unwrap();

        let packet = Unsuback::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Unsuback(decoded_unsuback) => {
                assert_eq!(unsuback, decoded_unsuback)
            }
            _ => panic!("Expected a Puback packet but got something else"),
        }
    }
}
