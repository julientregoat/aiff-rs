mod ids;

use bytes::buf::Buf;
use rust_decimal::Decimal;
use std::io::Cursor;
use self::ids::*;

type Buffer<'a> = &'a mut Cursor<Vec<u8>>;

// The first 8 bytes of a chunk are chunk ID and chunk size
pub struct ChunkBuilder(ID);

// The 'parse' fns could return a dynamic type
// but this allows us to lean on type checking, right?
// otherwise we still have to figure out what type is returned anyway
impl ChunkBuilder {
    pub fn new(buffer: Buffer) -> ChunkBuilder {
        let mut id = [0; 4];
        buffer.copy_to_slice(&mut id);
        ChunkBuilder(id)
    }

    pub fn id(&self) -> &ID {
        let ChunkBuilder(id) = &self;
        id
    }

    pub fn consume(self) -> ID {
        let ChunkBuilder(id) = self;
        id
    }
}

#[derive(Debug)]
pub enum ChunkError {
    InvalidID(ID),
    InvalidFormType,
    InvalidID3Version([u8; 2])
}

pub trait Chunk {
    fn build(cb: ChunkBuilder, buffer: Buffer) -> Result<Self, ChunkError>
        where
            Self: Sized;
}

// container for all other chunks in file
// TODO return CompleteFormChunk when CommonChunk is present?
// assuming we can't rely on the common chunk being at the start
pub struct FormChunk {
    size: i32,
    common: Option<CommonChunk>,
    sound: Option<SoundDataChunk>,
    chunks: Vec<Box<dyn Chunk>>,
}

impl FormChunk {
    fn add_common(&mut self, chunk: CommonChunk) {
        self.common = Some(chunk);
    }

    fn add_sound(&mut self, chunk: SoundDataChunk) {
        self.sound = Some(chunk);
    }

    fn add_chunk(&mut self, chunk: Box<dyn Chunk>) {
        self.chunks.push(chunk);
    }

    pub fn load_chunks(&mut self, buf: Buffer) -> Result<(), ChunkError> {
        // FIXME don't go over index
        loop {
            let cb = ChunkBuilder::new( buf);

            // once the common and form are detected, we can loop
            match cb.id() {
                COMMON => {
                    println!("Common chunk detected");
                    let common = CommonChunk::build(cb, buf).unwrap();
                    println!(
                        "frames {} size {} rate {}",
                        common.num_sample_frames, common.sample_size, common.sample_rate
                    );
                    self.add_common(common);

                },
                SOUND => {
                    println!("SOUND chunk detected");
                    let sound = SoundDataChunk::build(cb, buf).unwrap();
                    println!("size {} offset {} block size {}", sound.size, sound.offset, sound.block_size);
                    self.add_sound(sound);
                },
                MARKER => println!("MARKER chunk detected"),
                INSTRUMENT => println!("INSTRUMENT chunk detected"),
                MIDI => println!("MIDI chunk detected"),
                RECORDING => println!("RECORDING chunk detected"),
                APPLICATION => println!("APPLICATION chunk detected"),
                COMMENT => println!("COMMENT chunk detected"),
                NAME | AUTHOR | COPYRIGHT | ANNOTATION => {
                    let text = TextChunk::build(cb, buf).unwrap();
                    println!("TEXT chunk detected: {}", text.text);
                    self.add_chunk(Box::new(text));
                },
                FVER=> {println!("FVER chunk detected"); unimplemented!();},
                // 3 bytes "ID3" identifier. 4th byte is first version byte
                [73, 68, 51, _x] => {
                    match ID3Chunk::build(cb, buf) {
                        Ok(chunk) => self.add_chunk(Box::new(chunk)),
                        Err(e) => println!("Build ID3 chunk failed {:?}", e)
                    }
                },
                CHAN | BASC | TRNS | CATE => println!("apple stuff"),
                _ => (),
//                id => println!("other chunk {:?}", id),
            }
        }
    }
}

impl Chunk for FormChunk {
    fn build(cb: ChunkBuilder, buf: Buffer) -> Result<FormChunk, ChunkError> {
        if cb.id() != FORM {
            return Err(ChunkError::InvalidID(cb.consume()));
        }

        let size = buf.get_i32_be();
        let mut form_type = [0; 4];
        buf.copy_to_slice(&mut form_type);

        match &form_type {
            AIFF_C => {
                println!("aiff c file detected");
                Err(ChunkError::InvalidFormType)
            }
            AIFF => Ok(FormChunk { size, common: None, sound: None, chunks: vec![]}),
            _ => Err(ChunkError::InvalidFormType)
        }
    }
}

struct CommonChunk {
    pub size: i32,
    pub num_channels: i16,
    pub num_sample_frames: u32,
    pub sample_size: i16,     // AKA bit depth
    pub sample_rate: Decimal, // 80 bit extended floating pt num
}

impl Chunk for CommonChunk {
    fn build(cb: ChunkBuilder, buf: Buffer) -> Result<CommonChunk, ChunkError> {
        if cb.id() != COMMON {
            return Err(ChunkError::InvalidID(cb.consume()))
        }

        let (size, num_channels, num_sample_frames, sample_size) =
            (buf.get_i32_be(), buf.get_i16_be(), buf.get_u32_be(), buf.get_i16_be());

        // rust_decimal requires 96 bits to create a decimal
        // the extended precision / double long sample rate is 80 bits
        let mut rate_low = [0; 4];
        let mut rate_mid = [0; 4];
        let mut rate_hi = [0; 4];

        buf.copy_to_slice(&mut rate_hi[2..]);
        buf.copy_to_slice(&mut rate_mid);
        buf.copy_to_slice(&mut rate_low);

        let sample_rate = Decimal::from_parts(
            u32::from_le_bytes(rate_low),
            u32::from_le_bytes(rate_mid),
            u32::from_le_bytes(rate_hi),
            false,
            23,
        );

        Ok(CommonChunk {
            size,
            num_channels,
            num_sample_frames,
            sample_size,
            sample_rate,
        })
    }
}

struct SoundDataChunk {
    pub size: i32,
    pub offset: u32,
    pub block_size: u32,
    pub sound_data: Vec<u8>
}

impl Chunk for SoundDataChunk {
    fn build(cb: ChunkBuilder, buf: Buffer) -> Result<SoundDataChunk, ChunkError> {
        // A generic for the tag check would be nice
        if cb.id() != SOUND {
            return Err(ChunkError::InvalidID(cb.consume()));
        }

        let size = buf.get_i32_be();
        let offset = buf.get_u32_be();
        let block_size = buf.get_u32_be();
        let mut sound_data = vec![];

        // sound data length = chunk size (bytes) - 4byte offset - 4byte blocksize
        for _ in 8..size {
            sound_data.push(buf.get_u8());
        }

        Ok(SoundDataChunk {
            size,
            offset,
            block_size,
            sound_data
        })
    }
}

// TODO testme with pascal strings
pub fn read_pstring(buf: Buffer) -> String {
    let len = buf.get_u8();
    let mut str_buf = vec![];

    for _ in 0..len {
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
    Annotation,
}

struct TextChunk {
    chunk_type: TextChunkType,
    size: i32,
    text: String,
}

impl Chunk for TextChunk {
    fn build(cb: ChunkBuilder, buf: Buffer) -> Result<TextChunk, ChunkError> {
        let chunk_type = match cb.id() {
            NAME => TextChunkType::Name,
            AUTHOR => TextChunkType::Author,
            COPYRIGHT => TextChunkType::Copyright,
            ANNOTATION => TextChunkType::Annotation,
            _ => return Err(ChunkError::InvalidID(cb.consume())),
        };

        let size = buf.get_i32_be();
        let mut text_bytes = vec![];
        for _ in 0..size {
            text_bytes.push(buf.get_u8());
        }
        let text = String::from_utf8(text_bytes).unwrap();

        Ok(TextChunk {
            chunk_type,
            size,
            text,
        })
    }
}

struct ID3Chunk {
    version: [u8; 2]
}

impl Chunk for ID3Chunk {
    fn build(cb: ChunkBuilder, buf: Buffer) -> Result<ID3Chunk, ChunkError> {
        let id = cb.id();
        if id[0..3] != ID3[0..3] {
            return Err(ChunkError::InvalidID(cb.consume()))
        }

        let version = [id[3], buf.get_u8()];

        // constants?
        match version {
            [2, 0] => println!("id3 v2.0-2.2"),
            [3, 0] => println!("id3 v2.3"),
            [4, 0] => println!("id3 v2.4"),
            x => {
                println!("unknown version {:?}", x);
                return Err(ChunkError::InvalidID3Version(x))
            }
        }

        println!("ID3 version {:?}", version);

        Ok(ID3Chunk{ version })
    }
}