#[derive(Debug)]
pub enum MqttError {
    ConnectError,
    PingError,
    InvalidQos,
    InvalidTopic,
    ReadMessageError,
    PublishError,
    SubscribeError,
    UnsubscribeError,
    PubrelError,
}
