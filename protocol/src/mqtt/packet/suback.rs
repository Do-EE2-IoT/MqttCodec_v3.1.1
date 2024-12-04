use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};
use bytes::{Buf, BufMut, BytesMut};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;

#[derive(Debug, PartialEq)]
pub struct Suback {
    pub packet_id: u16,
    pub qos: u8,
}

impl Encode for Suback {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Puback as u8);
        buffer.put_u8(0x03);
        buffer.put_u16(self.packet_id);
        if self.qos > 2 {
            return Err(EncodeError::UnexpectedEndOfInput);
        }
        buffer.put_u8(self.qos);
        Ok(())
    }
}

impl Decode for Suback {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError> {
        println!("len = {}", buffer.len());
        if buffer.len() != 5 {
            Err(DecodeError::InvalidMessageFormat)
        } else {
            let header = buffer.get_u8();
            buffer.get_u8();
            let packet_id = buffer.get_u16();
            println!(
                "Get Suback packet 0x{:02x} , packet id = {:04x}",
                header, packet_id
            );
            let qos = buffer.get_u8();
            if qos > 2 {
                return Err(DecodeError::UnexpectedEndOfInput);
            }
            buffer.clear();
            Ok(Packet::Suback(Self { packet_id, qos }))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Suback;

    #[test]
    fn test_suback_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let suback_packet = Suback {
            packet_id: 1234,
            qos: 2,
        };
        suback_packet
            .encode(&mut buffer)
            .expect("must be encode suback packet success");
        let packet = Suback::decode(&mut buffer).expect("Must be decode success here");
        match packet {
            crate::mqtt::types::Packet::Suback(decoded_suback) => {
                assert_eq!(decoded_suback, suback_packet)
            }
            _ => panic!("Expected a Suback packet but got something else"),
        }
    }

    #[test]
    fn test_qos_invalid() {
        let mut buffer = BytesMut::new();
        let suback_packet = Suback {
            packet_id: 1234,
            qos: 3,
        };
        assert!(suback_packet.encode(&mut buffer).is_err());
    }
}
