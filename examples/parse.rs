extern crate aiff;

use aiff::reader::AiffReader;
use std::fs::File;

fn main() {
    if let Ok(file) = File::open("./data/wave.aif") {
        let mut reader = AiffReader::new(file);
        match reader.read() {
            Ok(_) => {
                println!("{:?}", reader.samples::<f32>())
            },
            Err(e) => {
                panic!("Failed to read file {:?}", e)
            }
        }
    } else {
        panic!("NOOO")
    }
}
