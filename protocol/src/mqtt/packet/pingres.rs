use bytes::{Buf, BufMut};

use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};

use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;

#[derive(Debug, PartialEq)]
pub struct Pingres;

impl Encode for Pingres {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Pingresp as u8);
        buffer.put_u8(0x00);
        Ok(())
    }
}

impl Decode for Pingres {
    fn decode(buffer: &mut bytes::BytesMut) -> Result<Packet, DecodeError> {
        if buffer.len() != 2 {
            Err(DecodeError::InvalidMessageFormat)
        } else {
            let header = buffer.get_u8();
            println!("Get Pingres 0x{:02x}", header);
            buffer.clear();
            Ok(Packet::Pingres(Self))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Pingres;

    #[test]
    fn test_pingres_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let pingres = Pingres;
        pingres.encode(&mut buffer).unwrap();

        let packet = Pingres::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Pingres(decoded_pingres) => {
                assert_eq!(decoded_pingres, pingres)
            }
            _ => panic!("Expected a Pingres Packet but got something else"),
        }
    }
}
