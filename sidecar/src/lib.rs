pub mod rpc;

pub use rpc::{
    encode_frame, handle_message, read_frame, serve, sidecar_version, PROTOCOL_VERSION,
};
