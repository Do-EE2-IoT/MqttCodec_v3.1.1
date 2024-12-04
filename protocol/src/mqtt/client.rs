use futures::{SinkExt, StreamExt};
use tokio_util::codec::Framed;

use super::{
    codec::mqttcodec::MqttCodec,
    error::mqtt_error::MqttError,
    packet::{
        self,
        connect::Connect,
        disconnect::Disconnect,
        pingreq::Pingreq,
        publish::{Last4BitsFixHeader, Publish},
        subscribe::Subscribe,
        unsubscribe::Unsubscribe,
    },
    types::Packet,
};
use crate::mqtt::packet::connack::ConnackReturnCode;
use tokio::net::TcpStream;
pub struct Client {
    frame: Framed<TcpStream, MqttCodec>,
    client_id: String,
    keep_alive: u16,
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

    // pub async fn check_connect_timeout(&mut self) -> Result<(), MqttError> {
    //     if let Some(Ok(message)) = self.frame.next().await {
    //         match message {
    //             Packet::Connack(connack) => {
    //                 println!()
    //             },
    //             _ => Err(MqttError::ConnectError),
    //         }
    //     }
    // }

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
            packet_id = Some(1);
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
            packet_id: 2,
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
            packet_id: 3,
            topic,
        });
        if let Err(e) = self.frame.send(unsubscribe_packet).await {
            println!("{:?}", e);
        }

        Ok(())
    }

    pub async fn wait_message(&mut self) {
        if let Some(Ok(message)) = self.frame.next().await {
            match message {
                Packet::Connack(connack) => {
                    println!("Get Connack from broker!");
                    match ConnackReturnCode::from_u8(connack.return_code) {
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

                Packet::Pingres(_pingres) => {
                    println!("Get Pingres from broker!");
                }
                Packet::Puback(_puback) => {
                    println!("Get Puback from broker!");
                }

                Packet::Suback(_suback) => {
                    println!("Get Suback from broker");
                }

                Packet::Unsuback(_unsuback) => {
                    println!("Get Unsuback from broker");
                }
                Packet::Pubrec(_pubrec) => {
                    println!("Get Pubrec from broker");
                }
                Packet::Pubcomp(_pubcomp) => {
                    println!("Get Pubcomp from broker");
                }
                Packet::Publish(publish) => {
                    println!("Get Publish message");
                    println!("Topic: {}", publish.topic_name);
                    println!("Payload: {}", publish.payload)
                }
                _ => println!(" This message is't belong to client, ignoring...."),
            }
        }
    }
}
