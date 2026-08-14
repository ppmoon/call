use std::io::{self, BufReader};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    call_sidecar::serve(BufReader::new(stdin.lock()), stdout.lock())
        .expect("sidecar JSON-RPC serve");
}
