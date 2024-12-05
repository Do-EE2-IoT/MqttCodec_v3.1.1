use bytes::{Buf, BufMut, BytesMut};

use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;

#[derive(Debug, PartialEq)]
pub struct Puback {
    pub packet_id: u16,
}

impl Encode for Puback {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Puback as u8);
        buffer.put_u8(0x02);
        buffer.put_u16(self.packet_id);
        Ok(())
    }
}

impl Decode for Puback {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError> {
        if buffer.len() != 4 {
            Err(DecodeError::InvalidMessageFormat)
        } else {
            let header = buffer.get_u8();
            println!("Get Puback packet : 0x{:02x}", header);
            buffer.get_u8();
            let packet_id = buffer.get_u16();
            buffer.clear();
            let puback_packet = Self { packet_id };
            Ok(Packet::Puback(puback_packet))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Puback;

    #[test]
    fn test_puback_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let puback = Puback { packet_id: 12344 };
        puback.encode(&mut buffer).unwrap();

        let packet = Puback::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Puback(decoded_puback) => {
                assert_eq!(puback, decoded_puback)
            }
            _ => panic!("Expected a Puback packet but got something else"),
        }
    }
}
