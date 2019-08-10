extern crate bytes;
//extern crate cpal;
extern crate rust_decimal;

mod chunks;

use std::fs::File;
use std::io::{Cursor, Read};
use crate::chunks::{FormChunk, ChunkBuilder, Chunk};

fn main() {
//        let mut f = File::open("./sample.aif").unwrap();
    let mut f = File::open("./devil.aiff").unwrap();
//            let mut f = File::open("./purp.aiff").unwrap();

    // TODO don't read all into memory - use read_exact w loop
    let mut buf = Cursor::new(vec![]);
    f.read_to_end(buf.get_mut()).unwrap();

    if let Ok(mut form) = FormChunk::build(ChunkBuilder::new(&mut buf), &mut buf) {
        form.load_chunks(&mut buf);
    } else {
        println!("unsupported first chunk");
    }
}
