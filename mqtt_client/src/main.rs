use std::time::Duration;

use protocol::mqtt::client::{self, Client};
use tokio::{self, time::sleep};

#[tokio::main]
async fn main() {
    let mut client = Client::config("127.0.0.1", 1885, "Nguyen Van Do", 60).await;
    client
        .connect()
        .await
        .expect("Must be connect to mqtt broker");
    client.subscribe("/hello", 0).await.unwrap();
    // client
    //     .publish("/hello", 0, "Hanoi university of science and technology", 0)
    //     .await
    //     .unwrap();

    tokio::select! {
        _ = client.wait_message()  => {

        sleep(Duration::from_secs(1)).await;

        }  ,
    }

    client
        .publish("/hello", 0, "Hanoi university of science and technology", 0)
        .await
        .unwrap();
    loop {
        tokio::select! {
            _ = client.wait_message()  => {

            }  ,
        }
    }
}
