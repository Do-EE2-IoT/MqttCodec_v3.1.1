use std::str::{from_utf8, Utf8Error};

use bytes::{self, Buf, BufMut, BytesMut};

pub fn encode_utf8(buffer: &mut BytesMut, string: &str) {
    buffer.put_u16(string.len() as u16);
    buffer.put_slice(string.as_bytes());
}

pub fn decode_utf8(buffer: &mut BytesMut) -> Result<String, Utf8Error> {
    let string_length = buffer.get_u16();
    let string = decode_utf8_with_length(buffer, string_length as usize)
        .expect("Can't decode utf8 with length");
    Ok(string)
}

pub fn decode_utf8_with_length(buffer: &mut BytesMut, size: usize) -> Result<String, Utf8Error> {
    let mut read_byte = vec![0u8; size];
    buffer.copy_to_slice(&mut read_byte);

    match from_utf8(&read_byte) {
        Ok(string) => Ok(string.to_string()),
        Err(e) => {
            eprintln!("Failed to decode UTF-8: {}", e);
            Err(e)
        }
    }

}

#[cfg(test)]
mod tests {
    use bytes::{self, BytesMut};

    use super::{decode_utf8, encode_utf8};

    #[test]
    fn encode_decode_utf8_test() {
        let string = "Hello world";
        let mut buffer = BytesMut::new();
        encode_utf8(&mut buffer, string);
        let decoded_string = decode_utf8(&mut buffer).unwrap();
        assert_eq!(decoded_string, string);
    }
}
