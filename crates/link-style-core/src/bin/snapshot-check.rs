use std::io::{self, Read};
fn main() {
    let mut bytes = Vec::new();
    io::stdin()
        .take(link_style_core::MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    let report = link_style_core::respond(&bytes);
    println!("{}", serde_json::to_string(&report).unwrap());
    if report["status"] != "complete" {
        std::process::exit(2);
    }
}
