extern crate bytes;
//extern crate cpal;
extern crate rust_decimal;

mod chunks;

use crate::chunks::reader::ChunkReader;
use std::fs::File;

fn main() {
    //     let mut f = File::open("./sample.aif").unwrap();
    let mut f = File::open("./devil.aiff").unwrap();
    //    let mut f = File::open("./purp.aiff").unwrap();

    let mut reader = ChunkReader::new(&mut f);
    reader.read();
}
