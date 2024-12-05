#[derive(Debug)]
pub enum MqttError {
    ConnectError,
    PingError,
    InvalidQos,
    InvalidTopic,
    ReadMessageError,
}
