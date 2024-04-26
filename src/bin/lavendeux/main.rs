use std::io::Read;

use lavendeux_parser::{value::Value, Lavendeux};

fn main() {
    let mut lav = Lavendeux::new();

    // Read stdin until EOF
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to read from stdin");

    // Parse the input
    println!("");
    match lav.run(&buffer) {
        Ok(v) => {
            if let Value::Array(v) = v {
                for v in v {
                    println!("{}", v);
                }
            } else {
                println!("{}", v);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
