use crate::mqtt::error::decode::DecodeError;
use crate::mqtt::error::encode::EncodeError;
use crate::mqtt::packet::connect::Connect;
use crate::mqtt::packet::pingreq::Pingreq;
use bytes::BytesMut;

use super::packet::connack::Connack;
use super::packet::disconnect::Disconnect;
use super::packet::pingres::Pingres;
use super::packet::puback::Puback;
use super::packet::pubcomp::Pubcomp;
use super::packet::publish::Publish;
use super::packet::pubrec::Pubrec;
use super::packet::pubrel::Pubrel;
use super::packet::suback::Suback;
use super::packet::subscribe::Subscribe;
use super::packet::unsuback::Unsuback;
use super::packet::unsubscribe::Unsubscribe;

pub trait Encode {
    fn encode(&self, buffer: &mut BytesMut) -> Result<(), EncodeError>;
}
pub trait Decode {
    fn decode(buffer: &mut BytesMut) -> Result<Packet, DecodeError>;
}
#[derive(Debug)]
pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Disconnect(Disconnect),
    Pingreq(Pingreq),
    Pingres(Pingres),
    Puback(Puback),
    Publish(Publish),
    Subscribe(Subscribe),
    Suback(Suback),
    Unsubscribe(Unsubscribe),
    Unsuback(Unsuback),
    Pubrec(Pubrec),
    Pubrel(Pubrel),
    Pubcomp(Pubcomp),
}
