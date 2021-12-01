extern crate aiff;

use aiff::reader::AiffReader;
use std::fs::File;

fn main() {
    if let Ok(file) = File::open("./data/wave.aif") {

    } else {

    }
}
