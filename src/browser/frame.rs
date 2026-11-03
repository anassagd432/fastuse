//! Chrome's native messaging framing: a 32-bit length in native byte order, then that many bytes
//! of UTF-8 JSON. Chrome accepts at most 1 MB from the host and sends at most 64 MiB to it.

use std::io::{self, Read, Write};

use serde_json::Value;

pub const MAX_TO_BROWSER: usize = 1 << 20;
pub const MAX_FROM_BROWSER: usize = 64 << 20;

/// Next message, or None when the browser closed the pipe.
pub fn read_frame(r: &mut impl Read) -> io::Result<Option<Value>> {
    let mut len = [0u8; 4];
    match r.read_exact(&mut len) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let n = u32::from_ne_bytes(len) as usize;
    if n > MAX_FROM_BROWSER {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "native message too large"));
    }
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf)?;
    serde_json::from_slice(&buf).map(Some).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn write_frame(w: &mut impl Write, v: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(v).map_err(io::Error::other)?;
    if body.len() > MAX_TO_BROWSER {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "message to the browser exceeds 1 MB"));
    }
    w.write_all(&(body.len() as u32).to_ne_bytes())?;
    w.write_all(&body)?;
    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn frames_round_trip_and_end_cleanly() {
        let mut buf = Vec::new();
        write_frame(&mut buf, &json!({"id":1,"method":"ping"})).unwrap();
        write_frame(&mut buf, &json!({"event":"hello"})).unwrap();
        let mut r = &buf[..];
        assert_eq!(read_frame(&mut r).unwrap().unwrap()["method"], "ping");
        assert_eq!(read_frame(&mut r).unwrap().unwrap()["event"], "hello");
        assert!(read_frame(&mut r).unwrap().is_none());
    }

    #[test]
    fn length_counts_bytes_not_characters() {
        let mut buf = Vec::new();
        write_frame(&mut buf, &json!({"t":"é"})).unwrap();
        let n = u32::from_ne_bytes(buf[..4].try_into().unwrap()) as usize;
        assert_eq!(n, buf.len() - 4);
        assert!(n > "{\"t\":\"e\"}".len());
    }

    #[test]
    fn oversized_message_to_browser_is_refused() {
        let big = json!({"x":"a".repeat(MAX_TO_BROWSER)});
        assert!(write_frame(&mut Vec::new(), &big).is_err());
    }
}
