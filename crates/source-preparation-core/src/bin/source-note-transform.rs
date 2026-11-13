use std::io::{self, Read};

fn main() {
    let mut input = Vec::new();
    if io::stdin()
        .take(source_preparation_core::MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut input)
        .is_err()
    {
        eprintln!("source-note-transform: stdin read failed");
        std::process::exit(2);
    }
    let result = source_preparation_core::respond(&input);
    println!("{result}");
    if result["status"] != "complete" {
        std::process::exit(2);
    }
}
