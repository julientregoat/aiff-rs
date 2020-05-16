use super::{
    chunks::{self, Chunk, FormChunk},
    ids,
};
use seek_bufread::BufReader;
use std::io::{Read, Seek, SeekFrom};

pub type Buffer<'a, Source> = &'a mut BufReader<Source>; // TODO

// aiff reader handle its own parsing
// store form chunk at top level
// expose a top level api

// TODO impl iterator
// cursor vs bufreader?
pub struct AiffReader<Source> {
    buf: BufReader<Source>, // TODO would be cleaner to support for impl Read
    form_chunk: Option<FormChunk>,
}

impl<S: Read + Seek> AiffReader<S> {
    pub fn new(source: S) -> AiffReader<S> {
        AiffReader {
            buf: BufReader::new(source),
            form_chunk: None,
        }
    }

    // TODO remove chunk builder, replace with a fn if even needed
    // use buf directly
    pub fn read(&mut self) -> Result<(), chunks::ChunkError> {
        let form_id = read_chunk_id(&mut self.buf);
        let mut form = FormChunk::build(&mut self.buf, form_id)?;
        while self.buf.available() >= 4 {
            // until form chunk is found, don't do anything else

            let mut id = [0; 4];
            self.buf.read_exact(&mut id).unwrap();

            // once the common and form are detected, we can loop
            // buffer position is right past the id
            match &id {
                ids::COMMON => {
                    println!("Common chunk detected");
                    let common =
                        chunks::CommonChunk::build(&mut self.buf, id).unwrap();
                    println!(
                        "channels {} frames {} bit rate {} sample rate {}",
                        common.num_channels,
                        common.num_sample_frames,
                        common.bit_rate,
                        common.sample_rate
                    );
                    form.set_common(common);
                }
                ids::SOUND => {
                    println!("SOUND chunk detected");
                    let sound =
                        chunks::SoundDataChunk::build(&mut self.buf, id)
                            .unwrap();
                    println!(
                        "size {} offset {} block size {}",
                        sound.size, sound.offset, sound.block_size
                    );
                    form.set_sound(sound);
                }
                ids::MARKER => println!("MARKER chunk detected"),
                ids::INSTRUMENT => println!("INSTRUMENT chunk detected"),
                ids::MIDI => println!("MIDI chunk detected"),
                ids::RECORDING => println!("RECORDING chunk detected"),
                ids::APPLICATION => println!("APPLICATION chunk detected"),
                ids::COMMENT => println!("COMMENT chunk detected"),
                ids::NAME | ids::AUTHOR | ids::COPYRIGHT | ids::ANNOTATION => {
                    let text =
                        chunks::TextChunk::build(&mut self.buf, id).unwrap();
                    println!("TEXT chunk detected: {}", text.text);
                    form.add_chunk(Box::new(text));
                }
                ids::FVER => {
                    println!("FVER chunk detected");
                    unimplemented!();
                }
                // 3 bytes "ID3" identifier
                // TODO merge both options
                [73, 68, 51, _] => {
                    self.buf.seek(SeekFrom::Current(-4)).unwrap();
                    match chunks::ID3v2Chunk::build(&mut self.buf, id) {
                        Ok(chunk) => form.add_chunk(Box::new(chunk)),
                        Err(e) => println!("Build ID3 chunk failed {:?}", e),
                    }
                }
                [_, 73, 68, 51] => {
                    self.buf.seek(SeekFrom::Current(-3)).unwrap();
                    match chunks::ID3v2Chunk::build(&mut self.buf, id) {
                        Ok(chunk) => form.add_chunk(Box::new(chunk)),
                        Err(e) => println!("Build ID3 chunk failed {:?}", e),
                    }
                }
                [84, 65, 71, _] => println!("v1 id3"), // "TAG_"
                [_, 84, 65, 71] => println!("v1 id3"), // "_TAG"
                ids::CHAN | ids::BASC | ids::TRNS | ids::CATE => {
                    println!("apple stuff detected")
                }
                id => println!(
                    "other chunk {:?} {:?}",
                    id,
                    String::from_utf8_lossy(id)
                ),
                // _ => (),
            };
        }
        self.form_chunk = Some(form);

        // FIXME handle remaining bytes
        println!("buffer complete {} byte(s) left", self.buf.available());

        Ok(())
    }
}

pub fn read_chunk_id<S: Read>(buf: Buffer<S>) -> ids::ChunkID {
    let mut id = [0; 4];
    buf.read_exact(&mut id).unwrap();
    id
}

pub fn read_u8<S: Read>(buf: Buffer<S>) -> u8 {
    let mut b = [0; 1];
    if let Err(e) = buf.read_exact(&mut b) {
        panic!("unable to read_u8 {:?}", e)
    }
    b[0]
}

pub fn read_u16_be<S: Read + Seek>(buf: Buffer<S>) -> u16 {
    let mut b = [0; 2];
    if let Err(e) = buf.read_exact(&mut b) {
        panic!("unable to read_u8 {:?}", e)
    }
    u16::from_be_bytes(b)
}

pub fn read_u32_be<S: Read + Seek>(buf: &mut BufReader<S>) -> u32 {
    let mut b = [0; 4];
    if let Err(e) = buf.read_exact(&mut b) {
        panic!("unable to read_i32_be {:?}", e)
    }
    u32::from_be_bytes(b)
}

pub fn read_i16_be<S: Read + Seek>(buf: &mut BufReader<S>) -> i16 {
    let mut b = [0; 2];
    if let Err(e) = buf.read_exact(&mut b) {
        panic!("unable to read_i32_be {:?}", e)
    }
    i16::from_be_bytes(b)
}

// pub fn read_i32_be<S: Read>(buf: Buffer<S>) -> u8 {
pub fn read_i32_be<S: Read + Seek>(buf: &mut BufReader<S>) -> i32 {
    let mut b = [0; 4];
    if let Err(e) = buf.read_exact(&mut b) {
        panic!("unable to read_i32_be {:?}", e)
    }
    i32::from_be_bytes(b)
}

// TODO testme with pascal strings
pub fn read_pstring<S: Read>(buf: Buffer<S>) -> String {
    let len = read_u8(buf);
    let mut str_buf = vec![0; len as usize];
    buf.read_exact(&mut str_buf).unwrap();

    String::from_utf8(str_buf).unwrap()
}
