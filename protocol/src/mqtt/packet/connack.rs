use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;
use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::types::{Decode, Encode, Packet};
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug)]
pub enum ConnackReturnCode {
    ConnectionAccept = 0x00,
    UnacceptableProtocolVersion = 0x01,
    IdentifierReject = 0x02,
    ServerUnavailable = 0x03,
    BadUsernameOrPassword = 0x04,
    NotAuthorized = 0x05,
    UnknownCode,
}

impl ConnackReturnCode {
    pub fn from_u8(code: u8) -> Self {
        match code {
            0x00 => Self::ConnectionAccept,
            0x01 => Self::UnacceptableProtocolVersion,
            0x02 => Self::IdentifierReject,
            0x03 => Self::ServerUnavailable,
            0x04 => Self::BadUsernameOrPassword,
            0x05 => Self::NotAuthorized,
            _ => Self::UnknownCode,
        }
    }
}
#[derive(Debug)]
pub struct Connack {
    pub return_code: u8,
}

impl Connack {
    pub fn handle_return_code(&self) {
        match ConnackReturnCode::from_u8(self.return_code) {
            ConnackReturnCode::BadUsernameOrPassword => {
                println!("Bad username or password.");
            }
            ConnackReturnCode::IdentifierReject => {
                println!("Client identifier rejected.");
            }
            ConnackReturnCode::ServerUnavailable => {
                println!("Server unavailable.");
            }
            ConnackReturnCode::UnknownCode => {
                println!("Unknown return code.");
            }
            ConnackReturnCode::UnacceptableProtocolVersion => {
                println!("Unacceptable protocol version.");
            }
            ConnackReturnCode::NotAuthorized => {
                println!("Not authorized.");
            }
            ConnackReturnCode::ConnectionAccept => {
                println!("Connection accepted.");
            }
        }
    }
}

impl Encode for Connack {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError> {
        buffer.put_u8(ControlPackets::Connack as u8);
        buffer.put_u8(0x02);
        buffer.put_u8(0x00);
        buffer.put_u8(self.return_code);
        Ok(())
    }
}

impl Decode for Connack {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError> {
        if buffer.len() < 4 {
            return Err(DecodeError::InvalidMessageFormat);
        }
        let header = buffer.get_u8();
        println!("Get Connack Packet : 0x{:02x}", header);
        let remain_length = buffer.get_u8();
        if remain_length < 2 {
            return Err(DecodeError::MalformedPacket);
        }
        let session_present = buffer.get_u8() & 0x01; // Lấy bit 0 của byte thứ 3
        println!("Session Present Flag: {}", session_present);
        let return_code = buffer.get_u8();
        let connack_packet = Self { return_code };
        println!("Successfully received Connack from broker");
        Ok(Packet::Connack(connack_packet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mqtt::types::Packet;
    use bytes::BytesMut;

    #[test]
    fn test_connack_encode_decode_roundtrip() {
        let connack_packet_original = Connack { return_code: 0 };

        let mut buffer = BytesMut::new();

        connack_packet_original.encode(&mut buffer).unwrap();

        let decoded_packet = Connack::decode(&mut buffer).unwrap();

        match decoded_packet {
            Packet::Connack(connack_packet_decoded) => {
                assert_eq!(
                    connack_packet_decoded.return_code,
                    connack_packet_original.return_code
                );
            }
            _ => panic!("Expected a ConnackPacket but got something else"),
        }

        assert_eq!(buffer.len(), 0);
    }
}
