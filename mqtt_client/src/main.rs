mod input;
use crate::input::{ConsoleInput, Input, InputUser};
use protocol::mqtt::types::Packet;
use protocol::mqtt::{client::Client, packet::suback::Suback};
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

async fn suback_timeout_handle(mut rx: Receiver<Suback>) {
    while let Some(suback_req) = rx.recv().await {
        println!(
            "Waiting broker response suback with packet id = {}",
            suback_req.packet_id
        );
        match timeout(Duration::from_secs(1), rx.recv()).await {
            Ok(Some(suback_response)) => {
                if suback_response == suback_req {
                    println!(
                        "Get Suback from broker with packet id = {}",
                        suback_response.packet_id
                    );
                }
            }
            Ok(None) => println!("???"),
            Err(_) => {
                println!("Must send subscribe message again");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx): (Sender<InputUser>, Receiver<InputUser>) = tokio::sync::mpsc::channel(1);
    let (tx_suback, rx_suback): (Sender<Suback>, Receiver<Suback>) = tokio::sync::mpsc::channel(2);

    tokio::spawn(console_input_handle(tx));
    tokio::spawn(suback_timeout_handle(rx_suback));

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
                            println!("Get Puback from broker with packet id = {}", puback.packet_id);
                        }
                        Packet::Suback(suback) => {
                            tx_suback.send(suback).await.expect("Channel send must success");
                        }
                        Packet::Unsuback(unsuback) => {
                            println!("Get Unsuback from broker with packet id = {}", unsuback.packet_id);
                        }
                        Packet::Pubrec(pubrec) => {
                            println!("Get Pubrec from broker with packet id = {}", pubrec.packet_id);
                            if let Err(e) = client.send_pubrel(pubrec).await {
                                println!("{:?}", e);
                            }
                        }
                        Packet::Pubcomp(pubcomp) => {
                            println!("Get Pubcomp from broker with packet id = {}", pubcomp.packet_id);
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
                    InputUser::Subscribe {
                        topic,
                        qos,
                    } => {
                        if let Err(e) = client.subscribe(topic.as_str(), qos).await {
                            println!("Client can't send subscribe to broker");
                            println!("{:?}", e);
                        }
                        let suback_req = Suback { packet_id: client.get_current_packet_id() - 1, qos };
                        tx_suback.send(suback_req).await.expect("Channel send must success");
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
