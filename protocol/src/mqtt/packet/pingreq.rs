use bytes::BufMut;

use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;

#[derive(Debug, PartialEq)]
pub struct Pingreq;
impl Encode for Pingreq {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Pingreq as u8);
        buffer.put_u8(0x00);
        Ok(())
    }
}
impl Decode for Pingreq {
    fn decode(buffer: &mut bytes::BytesMut) -> Result<Packet, DecodeError> {
        if buffer.len() != 2 {
            Err(DecodeError::InvalidMessageFormat)
        } else {
            let pingreq_packet = Self;
            Ok(Packet::Pingreq(pingreq_packet))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mqtt::types::Packet;
    use bytes::BytesMut;
    #[test]
    fn test_pingreq_encode_decode_round_trip() {
        let pingreq_packet = Pingreq;
        let mut buffer = BytesMut::new();
        pingreq_packet.encode(&mut buffer).unwrap();
        let decoded_pingreq = Pingreq::decode(&mut buffer).unwrap();
        match decoded_pingreq {
            Packet::Pingreq(decoded_pingreq_packet) => {
                assert_eq!(pingreq_packet, decoded_pingreq_packet)
            }
            _ => panic!("Expected a Pingreq packet but got something else"),
        }
    }
}
