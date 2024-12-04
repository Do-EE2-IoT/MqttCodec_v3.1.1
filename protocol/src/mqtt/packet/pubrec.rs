use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};
use bytes::{Buf, BufMut, BytesMut};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;

#[derive(Debug, PartialEq)]
pub struct Pubrec {
    pub packet_id: u16,
}

impl Encode for Pubrec {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Pubrec as u8);
        buffer.put_u8(0x02);
        buffer.put_u16(self.packet_id);
        Ok(())
    }
}

impl Decode for Pubrec {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError> {
        if buffer.len() != 4 {
            Err(DecodeError::InvalidMessageFormat)
        } else {
            let header = buffer.get_u8();
            println!("Get Pubrec packet : 0x{:02x}", header);
            buffer.get_u8();
            let packet_id = buffer.get_u16();
            buffer.clear();
            let pubrec_packet = Self { packet_id };
            Ok(Packet::Pubrec(pubrec_packet))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Pubrec;

    #[test]
    fn test_pubrec_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let pubrec = Pubrec { packet_id: 12344 };
        pubrec.encode(&mut buffer).unwrap();

        let packet = Pubrec::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Pubrec(decoded_pubrec) => {
                assert_eq!(pubrec, decoded_pubrec);
            }
            _ => panic!("Expected a Puback packet but got something else"),
        }
    }
}
