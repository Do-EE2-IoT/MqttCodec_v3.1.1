mod input;
use crate::input::{ConsoleInput, Input, InputUser};
use protocol::mqtt::client::Client;
use protocol::mqtt::types::Packet;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::timeout;

async fn console_input_handle(tx: Sender<InputUser>) {
    let mut console_input = ConsoleInput {
        buffer: String::new(),
    };
    while let Ok(data) = console_input.pop().await {
        if let Err(e) = tx.send(data).await {
            println!("Can't use channel because of error {e}");
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx): (Sender<InputUser>, Receiver<InputUser>) = tokio::sync::mpsc::channel(1);

    tokio::spawn(console_input_handle(tx));

    let mut client = Client::new("127.0.0.1", 1885, "Nguyen Van Do", 60).await;
    client
        .connect()
        .await
        .expect("Must be connect to mqtt broker");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                client.ping().await.unwrap();
            },
            message = client.wait_message() => {
                match message {
                    Ok(packet) => match packet {
                        Packet::Connack(connack) => {
                            println!("Get Connack from broker!");
                            connack.handle_return_code();
                        }
                        Packet::Pingres(_pingres) => {
                            println!("Get Pingres from broker!");
                        }
                        Packet::Puback(puback) => {
                            // QoS 1 ACK: resolve caller đang chờ
                            client.resolve_ack(puback.packet_id, Packet::Puback(puback));
                        }
                        Packet::Suback(suback) => {
                            // Subscribe ACK: resolve caller đang chờ
                            client.resolve_ack(suback.packet_id, Packet::Suback(suback));
                        }
                        Packet::Unsuback(unsuback) => {
                            println!("Get Unsuback from broker with packet id = {}", unsuback.packet_id);
                        }
                        Packet::Pubrec(pubrec) => {
                            // QoS 2 step 2: tự động gửi PUBREL, không cần pending_ack
                            if let Err(e) = client.send_pubrel(pubrec).await {
                                println!("{:?}", e);
                            }
                        }
                        Packet::Pubcomp(pubcomp) => {
                            // QoS 2 final ACK: resolve caller đang chờ
                            client.resolve_ack(pubcomp.packet_id, Packet::Pubcomp(pubcomp));
                        }
                        Packet::Publish(publish) => {
                            println!("Get Publish message");
                            println!("Topic: {}", publish.topic_name);
                            println!("Payload: {}", publish.payload);
                        }
                        _ => println!("This message is't belong to client, ignoring...."),
                    },
                    Err(e) => println!("{:?}", e),
                }
            },
            Some(input) = rx.recv() => {
                match input {
                    InputUser::Publish {
                        topic,
                        qos,
                        message,
                    } => {
                        if let Err(e) = client.publish(&topic, qos, &message, 0).await {
                            println!("Client send publish fail");
                            println!("{:?}", e);
                        }
                    },
                    InputUser::Ping => {
                        if let Err(e) = client.ping().await {
                            println!("Client can't send ping to broker");
                            println!("{:?}", e);
                        }
                    },
                    InputUser::Subscribe { topic, qos } => {
                        match client.subscribe(&topic, qos).await {
                            Ok(rx_ack) => {
                                tokio::spawn(async move {
                                    match timeout(Duration::from_secs(5), rx_ack).await {
                                        Ok(Ok(Packet::Suback(suback))) => {
                                            println!("\u{2713} Subscribed OK, packet_id={}", suback.packet_id);
                                        }
                                        Ok(Ok(_)) => println!("Unexpected ACK type"),
                                        Ok(Err(_)) => println!("Sender dropped before ACK"),
                                        Err(_)     => println!("\u{2717} SUBACK timeout"),
                                    }
                                });
                            }
                            Err(e) => {
                                println!("Client can't send subscribe to broker");
                                println!("{:?}", e);
                            }
                        }
                    },
                    InputUser::Disconnect => {
                        println!("Disconnect");
                        if let Err(e) = client.disconnect().await {
                            println!("Client can't send disconnect to broker");
                            println!("{:?}", e);
                        }
                    },
                    InputUser::Unsubscribe {
                        topic,
                    } => {
                        if let Err(e) = client.unsubscribe(topic).await {
                            println!("Client can't send unsubscribe to broker");
                            println!("{:?}", e);
                        }
                    },
                }
            }
        }
    }
}
