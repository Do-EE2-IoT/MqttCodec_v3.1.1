use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub enum ControlPackets {
    Connect = 0b0001_0000,
    Connack = 0b0010_0000,
    Publish = 0b0011_0000,
    Puback = 0b0100_0000,
    Pubrec = 0b0101_0000,
    Pubrel = 0b0110_0010,
    Pubcomp = 0b0111_0000,
    Subscribe = 0b1000_0010,
    Suback = 0b1001_0000,
    Unsubscribe = 0b1010_0010,
    Unsuback = 0b1011_0000,
    Pingreq = 0b1100_0000,
    Pingresp = 0b1101_0000,
    Disconnect = 0b1110_0000,
}

impl TryFrom<u8> for ControlPackets {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b0001_0000 => Ok(Self::Connect),
            0b0010_0000 => Ok(Self::Connack),
            0b0011_0000 => Ok(Self::Publish),
            0b0100_0000 => Ok(Self::Puback),
            0b0101_0000 => Ok(Self::Pubrec),
            0b0110_0010 => Ok(Self::Pubrel),
            0b0111_0000 => Ok(Self::Pubcomp),
            0b1000_0010 => Ok(Self::Subscribe),
            0b1001_0000 => Ok(Self::Suback),
            0b1010_0010 => Ok(Self::Unsubscribe),
            0b1011_0000 => Ok(Self::Unsuback),
            0b1100_0000 => Ok(Self::Pingreq),
            0b1101_0000 => Ok(Self::Pingresp),
            0b1110_0000 => Ok(Self::Disconnect),
        
            _ => Err("Unknown Control Packet"),
        }
    }
}
