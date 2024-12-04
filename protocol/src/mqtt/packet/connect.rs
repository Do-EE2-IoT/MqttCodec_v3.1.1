use crate::mqtt::types::Packet;
use crate::mqtt::types::{Decode, Encode};
use crate::mqtt::utils;
use crate::mqtt::{
    error::decode::DecodeError, error::encode::EncodeError, fix_header::ControlPackets,
};
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, PartialEq)]
pub struct Connect {
    pub keep_alive: u16,
    pub client_id: String,
}

impl Encode for Connect {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError> {
        if !self.client_id.is_empty() {
            buffer.put_u8(ControlPackets::Connect as u8);
            let remaining_length = 10 + 2 + self.client_id.len();
            buffer.put_u8(remaining_length as u8);
            utils::encode_utf8(buffer, "MQTT");
            buffer.put_u8(0x04);
            buffer.put_u8(0x02);
            buffer.put_u16(self.keep_alive);
            utils::encode_utf8(buffer, &self.client_id);
            Ok(())
        } else {
            Err(EncodeError::UnexpectedEndOfInput)
        }
    }
}

impl Decode for Connect {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError> {
        let header = buffer.get_u8();
        println!("Get Connect packet 0x{:02x}", header);
        buffer.get_u8();
        let protocol_name = utils::decode_utf8(buffer).expect("Must be decode utf8 success");
        println!("Protocol : {}", protocol_name);
        buffer.get_u16();
        let keep_alive = buffer.get_u16();
        let client_id = utils::decode_utf8(buffer).expect("Must be decode utf8 success");
        println!("Client ID 0x{} , keep alive = {}", client_id, keep_alive);
        buffer.clear();
        Ok(Packet::Connect(Self {
            keep_alive,
            client_id,
        }))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::mqtt::types::{Decode, Encode};

    use super::Connect;
    #[test]
    fn test_connect_encode_decode_round_trip() {
        let mut buffer = BytesMut::new();
        let connect_packet = Connect {
            keep_alive: 60,
            client_id: "Nguyen Van Do".to_string(),
        };
        connect_packet.encode(&mut buffer).unwrap();

        let packet = Connect::decode(&mut buffer).unwrap();
        match packet {
            crate::mqtt::types::Packet::Connect(decoded_connect) => {
                assert_eq!(decoded_connect, connect_packet)
            }
            _ => panic!("Expected a ConnackPacket but got something else"),
        }
    }
}
