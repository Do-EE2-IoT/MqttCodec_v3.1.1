use bytes::{Buf, BufMut};

use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;

#[derive(Debug, PartialEq)]
pub struct Disconnect;

impl Encode for Disconnect {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Disconnect as u8);
        buffer.put_u8(0x00);
        Ok(())
    }
}

impl Decode for Disconnect {
    fn decode(buffer: &mut bytes::BytesMut) -> Result<Packet, DecodeError> {
        if buffer.len() != 2 {
            Err(DecodeError::InvalidMessageFormat)
        } else {
            let header = buffer.get_u8();
            println!("Get Disconnect Packet 0x{:02x}", header);
            let disconnect_packet = Self;
            Ok(Packet::Disconnect(disconnect_packet))
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Disconnect;

    #[test]
    fn test_disconnect_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let disconnect_packet = Disconnect;
        disconnect_packet.encode(&mut buffer).unwrap();

        let packet_decoded = Disconnect::decode(&mut buffer).unwrap();
        match packet_decoded {
            crate::mqtt::types::Packet::Disconnect(disconnect_packet_decoded) => {
                assert_eq!(disconnect_packet, disconnect_packet_decoded)
            }
            _ => panic!("Expected a ConnackPacket but got something else"),
        }
    }
}
