use futures::{SinkExt, StreamExt, TryFutureExt};
use std::collections::HashMap;
use tokio::sync::oneshot;
use tokio_util::codec::Framed;

use super::{
    codec::mqttcodec::MqttCodec,
    error::mqtt_error::MqttError,
    packet::{
        connect::Connect,
        disconnect::Disconnect,
        pingreq::Pingreq,
        publish::{Last4BitsFixHeader, Publish},
        pubrec::Pubrec,
        subscribe::Subscribe,
        unsubscribe::Unsubscribe,
    },
    types::Packet,
};
use crate::mqtt::packet::pubrel::Pubrel;
use tokio::net::TcpStream;
pub struct Client {
    frame: Framed<TcpStream, MqttCodec>,
    client_id: String,
    keep_alive: u16,
    current_packet_id: u16,
    pending_ack: HashMap<u16, oneshot::Sender<Packet>>,
}

impl Client {
    pub fn get_current_packet_id(&self) -> u16 {
        self.current_packet_id
    }

    pub fn set_current_packet_id(&mut self, packet_id: u16) {
        self.current_packet_id = packet_id;
    }

    pub async fn new(host: &str, port: u16, client_id: &str, keep_alive: u16) -> Self {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(addr)
            .await
            .expect("Can't init connection with broker");
        let frame = Framed::new(stream, MqttCodec);
        Self {
            frame,
            client_id: client_id.to_string(),
            keep_alive,
            current_packet_id: 1,
            pending_ack: HashMap::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), MqttError> {
        let connect_pkg: Packet = Packet::Connect(Connect {
            keep_alive: self.keep_alive,
            client_id: self.client_id.clone(),
        });
        self.frame
            .send(connect_pkg)
            .map_err(|_| MqttError::ConnectError)
            .await?;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), MqttError> {
        let disconnect_pkg = Packet::Disconnect(Disconnect);
        self.frame
            .send(disconnect_pkg)
            .map_err(|_| MqttError::ConnectError)
            .await?;
        Ok(())
    }

    pub async fn ping(&mut self) -> Result<(), MqttError> {
        let ping_packet = Packet::Pingreq(Pingreq);
        self.frame
            .send(ping_packet)
            .map_err(|_| MqttError::PingError)
            .await?;
        Ok(())
    }

    pub async fn publish(
        &mut self,
        topic: &str,
        qos: u8,
        payload: &str,
        retain: u8,
    ) -> Result<(), MqttError> {
        if topic.is_empty() {
            return Err(MqttError::InvalidTopic);
        }
        if qos > 2 {
            return Err(MqttError::InvalidQos);
        }
        let mut packet_id: Option<u16> = None;
        if qos > 0 {
            packet_id = Some(self.next_packet_id());
        }

        let publish_packet = Packet::Publish(Publish {
            last4bits_fix_header: Last4BitsFixHeader {
                dup_flag: 0,
                qos_level: qos,
                retain,
            },
            topic_name: topic.to_string(),
            packet_id,
            payload: payload.to_string(),
        });
        self.frame
            .send(publish_packet)
            .map_err(|_| MqttError::PublishError)
            .await?;
        Ok(())
    }

    pub async fn subscribe(
        &mut self,
        topic: &str,
        qos: u8,
    ) -> Result<oneshot::Receiver<Packet>, MqttError> {
        if topic.is_empty() {
            return Err(MqttError::InvalidTopic);
        }
        if qos > 2 {
            return Err(MqttError::InvalidQos);
        }
        // Lưu packet_id vào biến TRƯỚC khi tạo packet
        // next_packet_id() tăng counter ngay sau khi trả, nên phải dùng biến trung gian
        let packet_id = self.next_packet_id();
        let subscribe_packet = Packet::Subscribe(Subscribe {
            packet_id,
            topic: topic.to_string(),
            qos,
        });
        let (tx, rx): (oneshot::Sender<Packet>, oneshot::Receiver<Packet>) = oneshot::channel();
        self.pending_ack.insert(packet_id, tx); // key khớp với packet đã gửi
        self.frame
            .send(subscribe_packet)
            .map_err(|_| MqttError::SubscribeError)
            .await?;

        Ok(rx)
    }

    pub async fn unsubscribe(&mut self, topic: String) -> Result<(), MqttError> {
        let unsubscribe_packet = Packet::Unsubscribe(Unsubscribe {
            packet_id: self.next_packet_id(),
            topic,
        });
        self.frame
            .send(unsubscribe_packet)
            .map_err(|_| MqttError::UnsubscribeError)
            .await?;

        Ok(())
    }

    pub async fn send_pubrel(&mut self, pubrec: Pubrec) -> Result<(), MqttError> {
        let pubrel_packet = Packet::Pubrel(Pubrel {
            packet_id: pubrec.packet_id,
        });
        self.frame
            .send(pubrel_packet)
            .map_err(|_| MqttError::PubrelError)
            .await?;
        Ok(())
    }

    pub async fn wait_message(&mut self) -> Result<Packet, MqttError> {
        if let Some(Ok(message)) = self.frame.next().await {
            Ok(message)
        } else {
            Err(MqttError::ReadMessageError)
        }
    }
    fn next_packet_id(&mut self) -> u16 {
        let id = self.current_packet_id;
        self.current_packet_id = if self.current_packet_id == 65535 {
            1
        } else {
            self.current_packet_id + 1
        };
        id
    }

    /// Gửi `packet` về cho caller đang await rx (oneshot) với key là `packet_id`.
    /// Dùng chung cho mọi loại ACK: Suback, Puback, Pubcomp, ...
    pub fn resolve_ack(&mut self, packet_id: u16, packet: Packet) {
        if let Some(tx) = self.pending_ack.remove(&packet_id) {
            let _ = tx.send(packet);
            // Nếu caller đã timeout và drop rx thì send() trả Err — bỏ qua là đúng
        } else {
            println!("Received unexpected ACK for packet_id={packet_id}");
        }
    }
}
