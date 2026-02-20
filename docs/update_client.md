# MQTT Client – Phân Tích Kiến Trúc & Đề Xuất Cải Thiện

> **Mục tiêu tài liệu:** Phân tích trung thực kiến trúc hiện tại của `mqtt_client` và `protocol` crate, sau đó đề xuất hướng cải thiện cụ thể để bạn tự luyện tập Rust ở mức chuyên nghiệp hơn.

---

## 1. Tổng Quan Workspace Hiện Tại

```
MqttCodec_v3.1.1/
├── protocol/          ← thư viện MQTT codec + logic client
│   └── src/mqtt/
│       ├── client.rs  ← struct Client (kết nối, gửi packet)
│       ├── codec/     ← MqttCodec (Encoder + Decoder)
│       ├── error/     ← MqttError, DecodeError, EncodeError
│       ├── fix_header.rs ← ControlPackets enum
│       ├── packet/    ← 14 loại packet (connect, publish, ...)
│       ├── types.rs   ← trait Encode, Decode, enum Packet
│       └── utils.rs   ← helper encode/decode UTF-8
└── mqtt_client/       ← binary sử dụng protocol
    └── src/
        ├── main.rs    ← event loop chính
        └── input.rs   ← đọc stdin, parse lệnh
```

---

## 2. Phân Tích Kiến Trúc Hiện Tại

### 2.1 Điểm Tốt (giữ nguyên hoặc phát triển tiếp)

| Điểm mạnh | Vị trí |
|---|---|
| Tách `protocol` thành crate riêng | workspace |
| Dùng `tokio_util::codec::Framed` — đúng idiom Rust async | `client.rs`, `mqttcodec.rs` |
| Enum `Packet` gom tất cả loại packet — type-safe dispatch | `types.rs` |
| Có `From<io::Error>` cho `DecodeError` | `error/decode.rs` |
| Có unit test encode/decode cho `Publish` | `publish.rs` |
| Dùng trait `Input` + `async_trait` — cơ hội mở rộng | `input.rs` |

### 2.2 Vấn Đề Kiến Trúc Cần Cải Thiện

#### ❶ `Client::config()` — sai tên, sai thiết kế
```rust
// Hiện tại: config() mà lại TCP connect luôn bên trong
pub async fn config(host: &str, port: u16, ...) -> Self {
    let stream = TcpStream::connect(addr).await.expect("Can't init...");
    ...
}
```
**Vấn đề:**
- Tên `config` không nói lên việc nó thực hiện TCP connect.
- Dùng `expect()` ở đây → panic khi không kết nối được, không trả lỗi về caller.
- Kiến trúc đúng là tách *cấu hình* khỏi *kết nối*.

**Đề xuất:**
```rust
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub keep_alive: u16,
}

impl Client {
    pub fn new(config: ClientConfig) -> Self { ... }         // chỉ lưu config
    pub async fn connect(&mut self) -> Result<(), MqttError> { ... } // mở TCP + gửi CONNECT
}
```

---

#### ❷ `pub current_packet_id` — vi phạm encapsulation
```rust
pub struct Client {
    pub current_packet_id: u16,  // ← đang public
}
```
**Vấn đề:** `main.rs` truy cập trực tiếp `client.current_packet_id - 1` để tái tạo `Suback` request. Đây là code mùi — caller phải biết quá nhiều về internal state của `Client`.

**Đề xuất:**
- Đổi `current_packet_id` thành `private`.
- `subscribe()` trả về `packet_id` đã dùng: `pub async fn subscribe(...) -> Result<u16, MqttError>`.

---

#### ❸ Event loop trong `main.rs` — quá nhiều trách nhiệm (God Function)

`main()` đang làm tất cả:
- Lắng nghe stdin
- Dispatch lệnh của user
- Xử lý mọi loại packet nhận từ broker
- Quản lý timeout SUBACK
- Gửi PUBREL khi nhận PUBREC (QoS 2 flow)

**Đề xuất tách thành handler riêng:**
```rust
// Tách handler nhận packet thành hàm riêng
async fn handle_incoming(packet: Packet, client: &mut Client, ...) {
    match packet {
        Packet::Connack(p) => on_connack(p),
        Packet::Publish(p) => on_publish(p),
        Packet::Pubrec(p)  => client.send_pubrel(p).await?,
        ...
    }
}
```

---

#### ❹ Xử lý lỗi — `println!` + `Ok(())` che giấu lỗi thật
```rust
pub async fn connect(&mut self) -> Result<(), MqttError> {
    if let Err(e) = self.frame.send(packet).await {
        println!("{:?}", e);   // ← lỗi bị nuốt!
    }
    Ok(())   // ← luôn trả Ok dù có lỗi
}
```
**Vấn đề:** Tất cả các hàm `connect`, `disconnect`, `ping`, `publish`, `subscribe`, `unsubscribe`, `send_pubrel` đều có pattern này — caller không bao giờ biết thực sự có lỗi hay không.

**Đề xuất:**
```rust
pub async fn connect(&mut self) -> Result<(), MqttError> {
    self.frame.send(packet).await
        .map_err(|_| MqttError::ConnectError)?;
    Ok(())
}
```

---

#### ❺ `MqttError` — quá nghèo, không có context

```rust
pub enum MqttError {
    ConnectError,
    PingError,
    InvalidQos,
    InvalidTopic,
    ReadMessageError,
}
```
**Vấn đề:**
- `ConnectError` không nói được thông tin gì về lý do thực sự.
- Thiếu variant cho các trường hợp như `BrokerDisconnected`, `Timeout`, lỗi encode/decode.
- Không implement `std::error::Error` → không thể dùng với chuỗi `?` và crate `thiserror`.

**Đề xuất dùng `thiserror`:**
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MqttError {
    #[error("Connection failed: {0}")]
    ConnectError(#[from] std::io::Error),

    #[error("Invalid QoS level: {0}")]
    InvalidQos(u8),

    #[error("Invalid topic: {0}")]
    InvalidTopic(String),

    #[error("Broker disconnected unexpectedly")]
    BrokerDisconnected,

    #[error("Encode error: {0}")]
    EncodeError(#[from] EncodeError),
}
```

---

#### ❻ `suback_timeout_handle` — logic sai về mặt protocol

```rust
async fn suback_timeout_handle(mut rx: Receiver<Suback>) {
    while let Some(suback_req) = rx.recv().await {
        // Nhận SUBACK "request" từ chính mình
        match timeout(Duration::from_secs(1), rx.recv()).await {
            Ok(Some(suback_response)) => {
                if suback_response == suback_req { ... }
```
**Vấn đề:** Đang dùng cùng 1 channel `rx` vừa để gửi "SUBACK mong đợi" vừa để nhận SUBACK thật từ broker. Điều này tạo ra race condition, logic khó theo dõi và dễ nhầm lẫn.

**Đề xuất:** Dùng pattern `pending_acks: HashMap<u16, oneshot::Sender<Suback>>` trong struct Client — khi gửi Subscribe thì đăng ký channel chờ, khi nhận Suback thì resolve channel đó.

---

#### ❼ `ControlPackets::try_from` — match cứng byte, không mask đúng cho Publish

```rust
// fix_header.rs
0b0011_0000 => Ok(Self::Publish),
```
**Vấn đề:** Byte đầu của PUBLISH có 4 bit thấp động (DUP, QoS, RETAIN). Vậy mà decode trong `mqttcodec.rs` match nguyên byte `src[0]` mà không mask trước → nếu nhận PUBLISH với QoS=1 (`0x32`) thì `try_from` sẽ trả về `Unknown Control Packet`.

**Đề xuất:** Mask trước khi match:
```rust
let first_byte = src[0];
let packet_type_byte = match first_byte >> 4 {
    // dùng nibble cao để nhận dạng loại packet
    3 => ControlPackets::Publish,
    ...
};
```
Hoặc đổi `ControlPackets::try_from` nhận `u8` sau khi đã AND với `0xF0`.

---

#### ❽ `remain_length` — encode sai với payload dài

```rust
// publish.rs
let remain_length = 2 + self.topic_name.len() + self.payload.len();
buffer.put_u8(remain_length as u8);  // ← chỉ 1 byte!
```
**Vấn đề:** MQTT spec định nghĩa Remaining Length có thể là 1-4 byte (Variable Length Encoding). Với payload > 200 byte thì 1 byte không đủ. Đây là lỗi nghiêm trọng về protocol compliance.

**Đề xuất thêm hàm encode remaining length:**
```rust
fn encode_remaining_length(buf: &mut BytesMut, mut length: usize) {
    loop {
        let mut byte = (length % 128) as u8;
        length /= 128;
        if length > 0 { byte |= 0x80; }
        buf.put_u8(byte);
        if length == 0 { break; }
    }
}
```

---

#### ❾ `payload: String` — giới hạn chỉ dùng UTF-8

```rust
pub struct Publish {
    pub payload: String,
    ...
}
```
**Vấn đề:** MQTT payload là `bytes`, không nhất thiết là text. Dùng `String` không hỗ trợ binary payload (ảnh, JSON, Protobuf...).

**Đề xuất:** Dùng `bytes::Bytes` hoặc `Vec<u8>` cho payload, tầng ứng dụng tự parse.

---

#### ❿ Không có Keep-alive timeout chủ động

Hiện tại PING được gửi theo `tokio::time::interval(60s)` trong vòng lặp `main`. Tuy nhiên nếu broker không trả lời PINGRESP trong thời gian cho phép, client không có cơ chế phát hiện — không có timeout handler nào cho PINGRESP.

---

## 3. Đề Xuất Kiến Trúc Mới

### 3.1 Sơ Đồ Tổng Quan

```
mqtt_client (binary)
├── main.rs              ← khởi tạo, wiring các component
├── app.rs               ← App struct: điều phối toàn bộ vòng đời
├── input/
│   ├── mod.rs
│   └── console.rs       ← ConsoleInput (giữ nguyên)
└── handler/
    ├── mod.rs
    └── packet_handler.rs ← xử lý packet nhận từ broker

protocol (library)
├── client/
│   ├── mod.rs
│   ├── client.rs        ← Client struct (sạch, không panic)
│   ├── config.rs        ← ClientConfig struct  [NEW]
│   └── pending_acks.rs  ← HashMap<packet_id, oneshot> [NEW]
├── codec/
│   └── mqttcodec.rs     ← sửa lỗi Publish mask
├── error/
│   └── mqtt_error.rs    ← dùng thiserror [REFACTOR]
├── packet/
│   └── ...              ← thêm remaining_length đúng spec
└── types.rs
```

### 3.2 Luồng Xử Lý Đề Xuất

```
ConsoleInput ──(mpsc)──► CommandDispatcher
                                │
                                ▼
                           Client::send_*(...)
                                │
                     ┌──────────┴──────────┐
                     ▼                     ▼
               TCP Frame              PendingAcks
             (tokio Framed)         (HashMap<u16, tx>)
                     │
                     ▼
              IncomingPacket
                     │
          ┌──────────┴──────────┐
          ▼                     ▼
    auto_handle             user_callback
  (QoS flow, ping)        (on_publish, on_suback)
```

---

## 4. Roadmap Tự Luyện Tập (Gợi Ý Thứ Tự)

| Bước | Nhiệm vụ | Kỹ năng Rust luyện được |
|------|----------|------------------------|
| 1 | Sửa `remain_length` encode đúng MQTT spec | Bit manipulation, `bytes::BufMut` |
| 2 | Sửa `ControlPackets::try_from` mask 4 bit cao | `From`/`TryFrom` trait |
| 3 | `Client::config()` → tách `ClientConfig` + sửa `connect()` trả lỗi đúng | Builder pattern, `Result` propagation |
| 4 | Đổi `current_packet_id` thành private, `subscribe()` trả `u16` | Encapsulation, API design |
| 5 | Đổi `MqttError` dùng `thiserror` + impl `std::error::Error` | Error handling idiom |
| 6 | Đổi `payload: String` → `bytes::Bytes` | Newtype pattern, zero-copy |
| 7 | Tách event loop `main()` thành `App::run()` + các hàm handler nhỏ | Separation of concerns |
| 8 | Implement `PendingAcks` với `oneshot` channel thay cho channel trick hiện tại | `tokio::sync::oneshot`, concurrent patterns |
| 9 | Thêm PINGRESP timeout (dùng `tokio::time::timeout`) | Timeout handling |
| 10 | Viết thêm unit test cho các packet còn lại | Test idioms trong Rust |

---

## 5. Tài Nguyên Tham Khảo

- [MQTT 3.1.1 Specification](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html) — đọc kỹ Section 2 (Packet format) và Section 3 (Remaining Length)
- [tokio tutorial](https://tokio.rs/tokio/tutorial) — đặc biệt phần `select!`, channels
- [bytes crate docs](https://docs.rs/bytes) — `Bytes`, `BytesMut`, `Buf`, `BufMut`
- [thiserror crate](https://docs.rs/thiserror) — error handling idiomatic Rust
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) — thiết kế public API
