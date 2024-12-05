use futures::{SinkExt, StreamExt};
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
    pub current_packet_id: u16,
}

impl Client {
    pub async fn config(host: &str, port: u16, client_id: &str, keep_alive: u16) -> Self {
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
        }
    }

    pub async fn connect(&mut self) -> Result<(), MqttError> {
        let connect_packet = Packet::Connect(Connect {
            keep_alive: self.keep_alive,
            client_id: self.client_id.clone(),
        });
        if let Err(e) = self.frame.send(connect_packet).await {
            println!("{:?}", e);
        }
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), MqttError> {
        let disconnect_packet = Packet::Disconnect(Disconnect);
        if let Err(e) = self.frame.send(disconnect_packet).await {
            println!("{:?}", e);
        }
        Ok(())
    }

    pub async fn ping(&mut self) -> Result<(), MqttError> {
        let ping_packet = Packet::Pingreq(Pingreq);
        if let Err(e) = self.frame.send(ping_packet).await {
            println!("{:?}", e);
        }
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
        if let Err(e) = self.frame.send(publish_packet).await {
            println!("{:?}", e);
        }
        Ok(())
    }

    pub async fn subscribe(&mut self, topic: &str, qos: u8) -> Result<(), MqttError> {
        if topic.is_empty() {
            return Err(MqttError::InvalidTopic);
        }
        if qos > 2 {
            return Err(MqttError::InvalidQos);
        }
        let subscribe_packet = Packet::Subscribe(Subscribe {
            packet_id: self.next_packet_id(),
            topic: topic.to_string(),
            qos,
        });
        if let Err(e) = self.frame.send(subscribe_packet).await {
            println!("{:?}", e);
        }

        Ok(())
    }

    pub async fn unsubscribe(&mut self, topic: String) -> Result<(), MqttError> {
        let unsubscribe_packet = Packet::Unsubscribe(Unsubscribe {
            packet_id: self.next_packet_id(),
            topic,
        });
        if let Err(e) = self.frame.send(unsubscribe_packet).await {
            println!("{:?}", e);
        }

        Ok(())
    }

    pub async fn send_pubrel(&mut self, pubrec: Pubrec) -> Result<(), MqttError> {
        let pubrel_packet = Packet::Pubrel(Pubrel {
            packet_id: pubrec.packet_id,
        });
        if let Err(e) = self.frame.send(pubrel_packet).await {
            println!("{:?}", e);
        }
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
}
