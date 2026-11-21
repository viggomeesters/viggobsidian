use activity_core::provenance::{MAX_REQUEST_BYTES, respond};
use std::io::{self, Read};

fn main() {
    let mut input = Vec::new();
    if io::stdin()
        .lock()
        .take(MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut input)
        .is_err()
    {
        eprintln!("activity-provenance-render: stdin read failed");
        std::process::exit(2);
    }
    let response = respond(&input);
    println!("{response}");
    if response["status"] != "complete" {
        std::process::exit(2);
    }
}
