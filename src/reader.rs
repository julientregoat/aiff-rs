use super::chunks::{Chunk, ChunkBuilder, FormChunk};
use std::io::{Cursor, Read};

// TODO iterator API, plus impl Read
pub struct AiffReader {
    buf: Cursor<Vec<u8>>,
}

impl AiffReader {
    pub fn new(source: &mut impl Read) -> AiffReader {
        // TODO don't read all into memory - use read_exact w loop
        let mut buf = Cursor::new(vec![]);
        source.read_to_end(buf.get_mut()).unwrap();

        AiffReader { buf }
    }

    pub fn read(&mut self) {
        // probably a way to make this look nicer
        if let Ok(mut form) =
            FormChunk::build(ChunkBuilder::new(&mut self.buf), &mut self.buf)
        {
            form.load_chunks(&mut self.buf);
        } else {
            println!("unsupported first chunk");
        }
    }
}
