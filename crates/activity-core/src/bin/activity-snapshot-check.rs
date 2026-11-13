use std::io::{self, Read};
fn main() {
    let mut bytes = Vec::new();
    io::stdin()
        .take(activity_core::MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    let result = activity_core::respond(&bytes);
    println!("{}", result);
    if result["status"] == "incomplete" {
        std::process::exit(2);
    }
}
