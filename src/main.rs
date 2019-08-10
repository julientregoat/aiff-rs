extern crate bytes;
//extern crate cpal;

use bytes::buf::Buf;
use std::fs::File;
use std::io::{Cursor, Read};

type ID = [u8; 4];
type Buffer<'a> = &'a mut Cursor<Vec<u8>>;
type Extended = [u8; 10]; // 80 bits

const FORM: &ID = b"FORM";
const AIFF: &ID = b"AIFF";
const COMMON: &ID = b"COMM";
const SOUND: &ID = b"SSND";
const MARKER: &ID = b"MARK";
const INSTRUMENT: &ID = b"INST";
const MIDI: &ID = b"MIDI";
const RECORDING: &ID = b"AESD";
const APPLICATION: &ID = b"APPL";
const COMMENT: &ID = b"COMT";
const NAME: &ID = b"NAME";
const AUTHOR: &ID = b"AUTH";
const COPYRIGHT: &ID = b"(c) ";
const ANNOTATION: &ID = b"ANNO";

// TODO remove pub usage

// The first 8 bytes of a chunk are chunk ID and chunk size
pub struct ChunkBuilder {
    id: ID,    // 4 char string - what's the best way to store this
    size: i32, // size of chunk data
}

pub enum ChunkError {
    InvalidID,
    InvalidFormType,
}

// The 'parse' fns could return a dynamic type
// but this allows us to lean on type checking, right?
// otherwise we still have to figure out what type is returned anyway
impl ChunkBuilder {
    pub fn new(buffer: Buffer) -> ChunkBuilder {
        let mut id = [0; 4];
        buffer.copy_to_slice(&mut id);
//        println!("new id {:?}", id);
        ChunkBuilder {
            id,
            size: buffer.get_i32_be(),
        }
    }

    pub fn id(&self) -> ID {
        self.id
    }
}

pub trait Chunk {
    fn build(cb: ChunkBuilder, buffer: Buffer) -> Result<Self, ChunkError>
    where
        Self: Sized;
}

// container for all other chunks in file
// TODO return CompleteFormChunk when CommonChunk is present?
// assuming we can't rely on the common chunk being at the start
struct FormChunk {
    pub size: i32,
    pub metadata: Option<CommonChunk>, // should this be stored separately?
                                       //    pub chunks: Vec<Box<dyn Chunk>>,
                                       //    pub chunks: Vec<i8> <-- this is how its specified, addrs?
}

impl FormChunk {
    pub fn add_common(&mut self, chunk: CommonChunk) {
        self.metadata = Some(chunk);
    }
}

impl Chunk for FormChunk {
    fn build(cb: ChunkBuilder, buf: Buffer) -> Result<FormChunk, ChunkError> {
        if &cb.id != FORM {
            return Err(ChunkError::InvalidFormType);
        }

        let mut form_type = [0; 4];
        buf.copy_to_slice(&mut form_type);

        if &form_type != AIFF {
            Err(ChunkError::InvalidFormType)
        } else {
            Ok(FormChunk {
                size: cb.size,
                metadata: None,
                //                chunks: vec![],
            })
        }
    }
}

struct CommonChunk {
    pub size: i32,
    pub num_channels: i16,
    pub num_sample_frames: u32,
    pub sample_size: i16, // AKA bit depth
                          //    pub sample_rate: Extended, // FIXME 80 bit extended floating pt num
}

impl Chunk for CommonChunk {
    fn build(cb: ChunkBuilder, buf: Buffer) -> Result<CommonChunk, ChunkError> {
        let (num_channels, num_sample_frames, sample_size) =
            (buf.get_i16_be(), buf.get_u32_be(), buf.get_i16_be());

        let mut sample_rate = [0; 10];
        buf.copy_to_slice(&mut sample_rate[0..10]);
        println!("sample rate {:?}", sample_rate);

        Ok(CommonChunk {
            size: cb.size,
            num_channels,
            num_sample_frames,
            sample_size,
            //            sample_rate,
        })
    }
}

struct SoundDataChunk {
    pub size: i32,
    pub offset: u32,
    pub block_size: u32,
    //    pub sound_data: Vec<u8>
}

pub fn read_pstring(buf: Buffer) -> String {
    let len = buf.get_u8();
    let mut str_buf = vec![];

    for i in 0..len {
        str_buf.push(buf.get_u8());
    }

    String::from_utf8(str_buf).unwrap()
}

struct Marker {
    id: i16,
    position: u32,
    marker_name: String,
}

struct MarkerChunk {
    pub size: i32,
    pub num_markers: u16,
    pub markers: Vec<Marker>,
}

enum TextChunkType {
    Name,
    Author,
    Copyright,
    Annotation
}

struct TextChunk {
    chunk_type: TextChunkType,
    size: i32,
    text: String
}

impl Chunk for TextChunk {
    fn build(cb: ChunkBuilder, buf: Buffer) -> Result<TextChunk, ChunkError> {
        let chunk_type = match &cb.id {
            NAME => TextChunkType::Name,
            AUTHOR => TextChunkType::Author,
            COPYRIGHT => TextChunkType::Copyright,
            ANNOTATION => TextChunkType::Annotation,
            _ => return Err(ChunkError::InvalidID)
        };
        let mut text_bytes = vec![];
        for i in 0..cb.size {
            text_bytes.push(buf.get_u8());
        }
        let text = String::from_utf8(text_bytes).unwrap();

        Ok(TextChunk { chunk_type, size: cb.size, text })
    }
}

fn main() {
    let mut f = File::open("./sample.aif").unwrap();
//        let mut f = File::open("./devil.aiff").unwrap();

    // TODO don't read all into memory - use read_exact w loop
    let mut buf = Cursor::new(vec![]);
    f.read_to_end(buf.get_mut()).unwrap();

    if let Ok(form) = FormChunk::build(ChunkBuilder::new(&mut buf), &mut buf) {
        loop {
            let chunk = ChunkBuilder::new(&mut buf);

            // once the common and form are detected, we can loop
            match &chunk.id {
                COMMON => {
                    println!("Common chunk detected");
                    if let Ok(common) = CommonChunk::build(chunk, &mut buf) {
                        println!(
                            "frames {} size {}",
                            common.num_sample_frames, common.sample_size
                        );
                    }
                }
                SOUND => println!("SOUND chunk detected"),
                MARKER => println!("MARKER chunk detected"),
                INSTRUMENT => println!("INSTRUMENT chunk detected"),
                MIDI => println!("MIDI chunk detected"),
                RECORDING => println!("RECORDING chunk detected"),
                APPLICATION => println!("APPLICATION chunk detected"),
                COMMENT => println!("COMMENT chunk detected"),
                NAME | AUTHOR | COPYRIGHT | ANNOTATION => {
                    println!("TEXT chunk detected");
                    if let Ok(text) = TextChunk::build(chunk, &mut buf) {
                        println!("TEXT: {}", text.text);
                    }
                },
//                _ => println!("unsupported chunk"),
                _ => (),
            }
        }
    } else {
        println!("unsupported first chunk");
    }
}
