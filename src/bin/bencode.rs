use bencode_rs::parse_bencode;
use std::io::{self, BufReader};

fn main() {
    let mut reader = BufReader::new(io::stdin());

    match parse_bencode(&mut reader) {
        Ok(Some(val)) => {
            println!("Parsed string: {}", val.to_string());
            println!("Generate bencode: {}", val.to_bencode());
        }
        Ok(None) => (),
        Err(e) => panic!("Error: {} ", e),
    }
}
