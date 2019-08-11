use crate::chunks::{Chunk, ChunkBuilder, FormChunk};
use std::io::{Cursor, Read};

// TODO  Should a stream be returned, or the entire data?
// We will know the full data size in advance.
// A stream would be more scalable but its importance depends on
// the performance implications of loading most AIFF files in full (let's say ~500mb)
pub struct ChunkReader {
    buf: Cursor<Vec<u8>>,
}

impl ChunkReader {
    pub fn new(source: &mut impl Read) -> ChunkReader {
        // TODO don't read all into memory - use read_exact w loop
        let mut buf = Cursor::new(vec![]);
        source.read_to_end(buf.get_mut()).unwrap();

        ChunkReader { buf }
    }

    pub fn read(&mut self) {
        if let Ok(mut form) = FormChunk::build(ChunkBuilder::new(&mut self.buf), &mut self.buf) {
            form.load_chunks(&mut self.buf);
        } else {
            println!("unsupported first chunk");
        }
    }
}
