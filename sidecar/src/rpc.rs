//! Sidecar JSON-RPC protocol: LSP-style `Content-Length` framing over stdio.

use serde_json::{json, Value};
use std::io::{BufRead, ErrorKind, Write};

/// Protocol version spoken by this Sidecar. Bump when the JSON-RPC shape changes.
pub const PROTOCOL_VERSION: u32 = 1;

pub fn sidecar_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn encode_frame(body: &str) -> Vec<u8> {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let mut out = Vec::with_capacity(header.len() + body.len());
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(body.as_bytes());
    out
}

pub fn read_frame(reader: &mut impl BufRead) -> std::io::Result<String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Err(std::io::Error::new(
                ErrorKind::UnexpectedEof,
                "eof while reading headers",
            ));
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        let Some((name, value)) = trimmed.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("content-length") {
            let parsed = value.trim().parse::<usize>().map_err(|e| {
                std::io::Error::new(ErrorKind::InvalidData, format!("Content-Length: {e}"))
            })?;
            content_length = Some(parsed);
        }
    }
    let len = content_length.ok_or_else(|| {
        std::io::Error::new(ErrorKind::InvalidData, "missing Content-Length header")
    })?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf)
        .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, format!("utf-8: {e}")))
}

/// Handle one JSON-RPC body. Returns `None` for notifications or unparseable input.
pub fn handle_message(body: &str) -> Option<String> {
    let req: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("sidecar: invalid json: {e}");
            return None;
        }
    };
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let id = match id {
        Some(id) if !id.is_null() => id,
        _ => return None,
    };

    let response = match method {
        "ping" | "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "version": sidecar_version(),
                "protocolVersion": PROTOCOL_VERSION,
            }
        }),
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("method not found: {method}"),
            }
        }),
    };
    Some(response.to_string())
}

pub fn serve(mut input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
    loop {
        let body = match read_frame(&mut input) {
            Ok(body) => body,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };
        if let Some(resp) = handle_message(&body) {
            output.write_all(&encode_frame(&resp))?;
            output.flush()?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn encode_frame_uses_byte_length() {
        let body = "{\"msg\":\"ping\"}";
        let frame = encode_frame(body);
        let as_str = String::from_utf8(frame).unwrap();
        assert!(as_str.starts_with(&format!("Content-Length: {}\r\n\r\n", body.len())));
        assert!(as_str.ends_with(body));
    }

    #[test]
    fn read_frame_roundtrips_encoded_body() {
        let body = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}";
        let frame = encode_frame(body);
        let mut cursor = Cursor::new(frame);
        assert_eq!(read_frame(&mut cursor).unwrap(), body);
    }

    #[test]
    fn ping_returns_crate_version_and_protocol() {
        let req = r#"{"jsonrpc":"2.0","id":7,"method":"ping","params":{}}"#;
        let raw = handle_message(req).expect("ping is a request");
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["id"], 7);
        assert_eq!(v["result"]["version"], sidecar_version());
        assert_eq!(v["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert!(v.get("error").is_none());
    }

    #[test]
    fn unknown_method_is_jsonrpc_method_not_found() {
        let req = r#"{"jsonrpc":"2.0","id":"abc","method":"explode"}"#;
        let raw = handle_message(req).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["id"], "abc");
        assert_eq!(v["error"]["code"], -32601);
    }

    #[test]
    fn notification_without_id_is_ignored() {
        let req = r#"{"jsonrpc":"2.0","method":"ping"}"#;
        assert!(handle_message(req).is_none());
    }

    #[test]
    fn serve_answers_ping_then_stops_on_eof() {
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        let input = encode_frame(req);
        let mut output = Vec::new();
        serve(Cursor::new(input), &mut output).unwrap();
        let mut cursor = Cursor::new(output);
        let body = read_frame(&mut cursor).unwrap();
        let v: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["result"]["version"], sidecar_version());
    }
}
