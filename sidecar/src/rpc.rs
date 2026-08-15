//! Sidecar JSON-RPC protocol: LSP-style `Content-Length` framing over stdio.

use crate::graph;
use crate::meta;
use crate::protocol::{BuildParams, Pin, RunArg};
use crate::python::index_workspace;
use crate::run::{self, NativeRunner};
use crate::scaffold;
use serde_json::{json, Value};
use std::io::{BufRead, ErrorKind, Write};
use std::path::{Path, PathBuf};

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

    let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
    let response = match dispatch(method, params) {
        Ok(result) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        }),
        Err((code, message)) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message },
        }),
    };
    Some(response.to_string())
}

fn dispatch(method: &str, params: Value) -> Result<Value, (i32, String)> {
    match method {
        "ping" | "initialize" => Ok(json!({
            "version": sidecar_version(),
            "protocolVersion": PROTOCOL_VERSION,
        })),
        "graph.build" => {
            let parsed: BuildParams =
                serde_json::from_value(params).map_err(|e| (-32602, e.to_string()))?;
            let graph = graph::build(&parsed).map_err(|e| (-32000, e))?;
            Ok(serde_json::to_value(graph).unwrap())
        }
        "graph.symbols" => {
            let root = req_str(&params, "workspaceRoot")?;
            let symbols = graph::list_symbols(&root).map_err(|e| (-32000, e))?;
            Ok(serde_json::to_value(symbols).unwrap())
        }
        "meta.load" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            Ok(serde_json::to_value(meta::load_meta(&root)).unwrap())
        }
        "meta.pin" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            let pin: Pin = serde_json::from_value(params).map_err(|e| (-32602, e.to_string()))?;
            Ok(serde_json::to_value(meta::upsert_pin(&root, pin).map_err(|e| (-32000, e))?).unwrap())
        }
        "meta.runArgs" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            let arg: RunArg = serde_json::from_value(params).map_err(|e| (-32602, e.to_string()))?;
            Ok(serde_json::to_value(meta::upsert_run_args(&root, arg).map_err(|e| (-32000, e))?).unwrap())
        }
        "meta.snapshot" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            Ok(serde_json::to_value(meta::snapshot_identities(&root).map_err(|e| (-32000, e))?).unwrap())
        }
        "meta.reconcile" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            let qn = req_str(&params, "qualifiedName")?;
            Ok(serde_json::to_value(meta::reconcile(&root, &qn).map_err(|e| (-32000, e))?).unwrap())
        }
        "meta.appendPrompt" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            let qn = req_str(&params, "qualifiedName")?;
            let prompt = req_str(&params, "prompt")?;
            let path = meta::append_prompt(&root, &qn, &prompt).map_err(|e| (-32000, e))?;
            Ok(json!({ "path": path }))
        }
        "edit.extract" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            let qn = req_str(&params, "qualifiedName")?;
            run::extract_function_source(&root, &qn).map_err(|e| (-32000, e))
        }
        "run.node" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            let qn = req_str(&params, "qualifiedName")?;
            let args = params
                .get("argsJson")
                .and_then(Value::as_str)
                .unwrap_or("{}");
            let python = python_from(&params);
            run::run_node(&NativeRunner, &root, &qn, args, &python).map_err(|e| (-32000, e))
                .map(|r| serde_json::to_value(r).unwrap())
        }
        "run.program" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            let entry = req_str(&params, "entry")?;
            let python = python_from(&params);
            run::run_program(&NativeRunner, &root, &entry, &python).map_err(|e| (-32000, e))
                .map(|r| serde_json::to_value(r).unwrap())
        }
        "scaffold.helloWorld" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            let language = params
                .get("language")
                .and_then(Value::as_str)
                .unwrap_or("python");
            let path = scaffold::scaffold(&root, language).map_err(|e| (-32000, e))?;
            Ok(json!({ "path": path }))
        }
        "scaffold.isEmpty" => {
            let root = PathBuf::from(req_str(&params, "workspaceRoot")?);
            Ok(json!({ "empty": scaffold::is_empty_of_source(&root) }))
        }
        "index.lookup" => {
            let root = req_str(&params, "workspaceRoot")?;
            let _ = index_workspace(Path::new(&root)).map_err(|e| (-32000, e))?;
            Ok(json!({ "ok": true }))
        }
        _ => Err((-32601, format!("method not found: {method}"))),
    }
}

fn req_str(params: &Value, key: &str) -> Result<String, (i32, String)> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| (-32602, format!("missing {key}")))
}

fn python_from(params: &Value) -> PathBuf {
    params
        .get("pythonPath")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(run::default_python)
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
