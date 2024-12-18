use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};
use bytes::{Buf, BufMut, BytesMut};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;

#[derive(Debug, PartialEq)]
pub struct Pubcomp {
    pub packet_id: u16,
}

impl Encode for Pubcomp {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Pubcomp as u8);
        buffer.put_u8(0x02);
        buffer.put_u16(self.packet_id);
        Ok(())
    }
}

impl Decode for Pubcomp {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError> {
        if buffer.len() != 4 {
            Err(DecodeError::InvalidMessageFormat)
        } else {
            let header = buffer.get_u8();
            println!("Get Pubcomp packet : 0x{:02x}", header);
            buffer.get_u8();
            let packet_id: u16 = buffer.get_u16();
            buffer.clear();
            let pubcomp_packet = Self { packet_id };
            Ok(Packet::Pubcomp(pubcomp_packet))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Pubcomp;

    #[test]
    fn test_pubrel_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let pubrel = Pubcomp { packet_id: 12344 };
        pubrel.encode(&mut buffer).unwrap();

        let packet = Pubcomp::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Pubcomp(decoded_pubcomp) => {
                assert_eq!(pubrel, decoded_pubcomp);
            }
            _ => panic!("Expected a Pubcomp packet but got something else"),
        }
    }
}
