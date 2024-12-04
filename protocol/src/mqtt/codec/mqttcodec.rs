use crate::mqtt::error::{decode::DecodeError, encode::EncodeError};
use crate::mqtt::fix_header::ControlPackets;
use crate::mqtt::packet::connack::Connack;
use crate::mqtt::packet::connect::Connect;
use crate::mqtt::packet::disconnect::Disconnect;
use crate::mqtt::packet::pingreq::Pingreq;
use crate::mqtt::packet::pingres::Pingres;
use crate::mqtt::packet::puback::Puback;
use crate::mqtt::packet::pubcomp::Pubcomp;
use crate::mqtt::packet::publish::Publish;
use crate::mqtt::packet::pubrec::Pubrec;
use crate::mqtt::packet::pubrel::Pubrel;
use crate::mqtt::packet::suback::Suback;
use crate::mqtt::packet::subscribe::Subscribe;
use crate::mqtt::packet::unsuback::Unsuback;
use crate::mqtt::packet::unsubscribe::Unsubscribe;
use crate::mqtt::types::{Decode, Encode, Packet};
use bytes::BytesMut;

use tokio_util::codec::{Decoder, Encoder};
pub struct MqttCodec;

impl Encoder<Packet> for MqttCodec {
    type Error = EncodeError;

    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            Packet::Connect(connect_packet) => connect_packet
                .encode(dst)
                .expect("Must be encode connect packet success"),
            Packet::Connack(connack_packet) => connack_packet
                .encode(dst)
                .expect("Must be encode connack packet success"),
            Packet::Disconnect(disconnect_packet) => disconnect_packet
                .encode(dst)
                .expect("Must be encode disconnect packet success"),
            Packet::Pingreq(pingreq_packet) => pingreq_packet
                .encode(dst)
                .expect("Must be encode pingreq packet success"),
            Packet::Pingres(pingres_packet) => pingres_packet
                .encode(dst)
                .expect("Must be encode pingres packet success"),
            Packet::Puback(puback_packet) => puback_packet
                .encode(dst)
                .expect("Must be encode puback packet success"),
            Packet::Publish(publish_packet) => publish_packet
                .encode(dst)
                .expect("Must be encode publish packet success"),
            Packet::Subscribe(subscribe_packet) => subscribe_packet
                .encode(dst)
                .expect("Must be encode subscribe packet success"),
            Packet::Suback(suback_packet) => suback_packet
                .encode(dst)
                .expect("Must be encode Suback packet success"),
            Packet::Unsubscribe(unsub_packet) => unsub_packet
                .encode(dst)
                .expect("Must be encode unsub packet success"),
            Packet::Unsuback(unsuback) => unsuback
                .encode(dst)
                .expect("Must be encode unsuback packet successfully"),
            Packet::Pubrec(pubrec) => pubrec
                .encode(dst)
                .expect("Must be encode Pubrec packet success"),
            Packet::Pubrel(pubrel) => pubrel
                .encode(dst)
                .expect("Must be encode pubrel packet success"),
            Packet::Pubcomp(pubcomp) => pubcomp
                .encode(dst)
                .expect("Must be encode pubcomp packet success"),
        }

        Ok(())
    }
}

impl Decoder for MqttCodec {
    type Item = Packet;

    type Error = DecodeError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        let first_byte = src[0];
        let header = if let Ok(byte) = ControlPackets::try_from(first_byte) {
            byte
        } else {
            println!("Get Unknown header from broker, ignore .....");
            return Err(DecodeError::MalformedPacket);
        };

        match header {
            ControlPackets::Connect => {
                let connect_packet = Connect::decode(src).expect("Must be decode Connect success");
                Ok(Some(connect_packet))
            }
            ControlPackets::Connack => {
                let connack_packet =
                    Connack::decode(src).expect("Must be decode Connack packet success");
                Ok(Some(connack_packet))
            }
            ControlPackets::Publish => {
                let publish_packet =
                    Publish::decode(src).expect("Must be decode Publish packet success");
                Ok(Some(publish_packet))
            }
            ControlPackets::Puback => {
                let puback_packet =
                    Puback::decode(src).expect("Must be decode Puback packet success");
                Ok(Some(puback_packet))
            }
            ControlPackets::Pubrec => {
                let pubrec_packet =
                    Pubrec::decode(src).expect("Must be decode Pubrec packet success");
                Ok(Some(pubrec_packet))
            }
            ControlPackets::Pubrel => {
                let pubrel_packet =
                    Pubrel::decode(src).expect("Must be decode Pubrel packet success");
                Ok(Some(pubrel_packet))
            }
            ControlPackets::Pubcomp => {
                let pubcomp_packet = Pubcomp::decode(src).expect("Must be decode Pubcomp success");
                Ok(Some(pubcomp_packet))
            }
            ControlPackets::Subscribe => {
                let subscribe_packet =
                    Subscribe::decode(src).expect("Must be decode for Subscribe success");
                Ok(Some(subscribe_packet))
            }
            ControlPackets::Suback => {
                let suback_packet = Suback::decode(src).expect("Must be decode for Suback success");
                Ok(Some(suback_packet))
            }
            ControlPackets::Unsubscribe => {
                let unsubscribe_packet =
                    Unsubscribe::decode(src).expect("Must be decode Unsubscribe packet success");
                Ok(Some(unsubscribe_packet))
            }
            ControlPackets::Unsuback => {
                let unsuback_packet =
                    Unsuback::decode(src).expect("Must be decode Unsuback packet success");
                Ok(Some(unsuback_packet))
            }
            ControlPackets::Pingreq => {
                let pingreq_packet =
                    Pingreq::decode(src).expect("Must be decode Pingreq packet success");
                Ok(Some(pingreq_packet))
            }
            ControlPackets::Pingresp => {
                let pingres_packet =
                    Pingres::decode(src).expect("Must be decode Pingres packet success");
                Ok(Some(pingres_packet))
            }
            ControlPackets::Disconnect => {
                let disconnect_packet =
                    Disconnect::decode(src).expect("Must be decode disconnect packet success)");
                Ok(Some(disconnect_packet))
            }
        }
    }
}
