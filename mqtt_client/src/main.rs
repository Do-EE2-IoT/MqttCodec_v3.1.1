mod input;
use crate::input::{ConsoleInput, Input, InputUser};
use protocol::mqtt::client::{self, Client};
use tokio;
use tokio::sync::mpsc::{Receiver, Sender};

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
    let mut client = Client::config("127.0.0.1", 1885, "Nguyen Van Do", 60).await;
    client
        .connect()
        .await
        .expect("Must be connect to mqtt broker");
    // client.subscribe("/hello", 0).await.unwrap();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    // client.wait_message().await;
    loop {
        tokio::select! {
            _ = interval.tick() => {
                client.ping().await.unwrap();
            },
            _ = client.wait_message()  => {} ,

            Some(input) = rx.recv() => {

                match input{
                    InputUser::Publish {
                        topic,
                        qos,
                        message,
                    } => {
                        if let Err(e) = client.publish(&topic, qos, &message, 0).await {
                            println!("Can't publish");
                        }
                    },
                    InputUser::Ping => {
                        if let Err(e) = client.ping().await {
                            println!("can;t ping");
                        }
                    },
                    InputUser::Subscribe {
                        topic,
                        qos,
                    } => {
                        if let Err(e) = client.subscribe(topic.as_str(), qos).await {
                            println!("Can't suv");
                        }
                    },
                    InputUser::Disconnect => {
                        println!("Disconnect");
                        if let Err(e) = client.disconnect().await {
                            println!("Cant disconnect");
                        }
                    },
                    InputUser::Unsubscribe {
                        topic,
                    } => {
                        if let Err(e) = client.unsubscribe(topic).await {
                            println!("can't unsub");
                        }
                    },

                }
            }

        }
    }
}
