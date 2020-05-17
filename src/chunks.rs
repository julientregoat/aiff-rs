use super::extended::parse_extended_precision_bytes;
use super::{
    ids::{self, ChunkID},
    reader::{self, Buffer},
};
use id3;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub enum ChunkError {
    InvalidID(ids::ChunkID),
    InvalidFormType(ids::ChunkID),
    InvalidID3Version([u8; 2]),
}

// TODO auto implemented 'check' fn with associated const ID
// auto impl '_build' that includes the check before calling inner build
pub trait Chunk {
    fn build<S: Read + Seek>(
        buffer: Buffer<S>,
        id: ChunkID,
    ) -> Result<Self, ChunkError>
    where
        Self: Sized;
}

// container for all other chunks in file
// TODO return CompleteFormChunk when CommonChunk + SoundChunk is present?
// assuming we can't rely on the common chunk being at the start
pub struct FormChunk {
    size: i32,
    common: Option<CommonChunk>,
    sound: Option<SoundDataChunk>,
    chunks: Vec<Box<dyn Chunk>>, // TODO no box dyn
}

impl FormChunk {
    pub fn common(&self) -> &Option<CommonChunk> {
        &self.common
    }

    pub fn set_common(&mut self, chunk: CommonChunk) {
        self.common = Some(chunk);
    }

    pub fn sound(&self) -> &Option<SoundDataChunk> {
        &self.sound
    }

    pub fn set_sound(&mut self, chunk: SoundDataChunk) {
        self.sound = Some(chunk);
    }

    pub fn chunks(&self) -> &Vec<Box<dyn Chunk>> {
        &self.chunks
    }

    pub fn add_chunk(&mut self, chunk: Box<dyn Chunk>) {
        self.chunks.push(chunk);
    }
}

impl Chunk for FormChunk {
    fn build<S: Read + Seek>(
        buf: Buffer<S>,
        id: ChunkID,
    ) -> Result<FormChunk, ChunkError> {
        if &id != ids::FORM {
            return Err(ChunkError::InvalidID(id));
        }
        // TODO validate this is correct chunk by id - rewind reader?

        let size = reader::read_i32_be(buf);
        println!("form chunk bytes {}", size);
        let mut form_type = [0; 4];
        buf.read_exact(&mut form_type).unwrap();

        match &form_type {
            ids::AIFF => Ok(FormChunk {
                size,
                common: None,
                sound: None,
                chunks: vec![],
            }),
            ids::AIFF_C => {
                println!("aiff c file detected; unsupported");
                Err(ChunkError::InvalidFormType(form_type))
            }
            &x => Err(ChunkError::InvalidFormType(x)),
        }
    }
}

pub struct CommonChunk {
    pub size: i32,
    pub num_channels: i16,
    pub num_sample_frames: u32,
    pub bit_rate: i16, // in the spec, this is defined as `sample_size`
    pub sample_rate: f64, // 80 bit extended floating pt num
}

impl Chunk for CommonChunk {
    fn build<S: Read + Seek>(
        buf: Buffer<S>,
        id: ChunkID,
    ) -> Result<CommonChunk, ChunkError> {
        if &id != ids::COMMON {
            return Err(ChunkError::InvalidID(id));
        }

        let (size, num_channels, num_sample_frames, bit_rate) = (
            reader::read_i32_be(buf),
            reader::read_i16_be(buf),
            reader::read_u32_be(buf),
            reader::read_i16_be(buf),
        );

        // FIXME this is broken
        // need to parse IEEE 754 80 bit extended precision
        // use crate simple_soft_float with FloatProperties
        // let props = simple_soft_float::FloatProperties::new_with_extended_flags(
        //     15,
        //     64,    // 1 bit integer part, 63 bit significand (???)
        //     false, // integer part != implicit leading bit (???)
        //     true,
        //     simple_soft_float::PlatformProperties::default(), // TODO
        // );

        // let mut rate_raw = [0; 10];
        // buf.read_exact(&mut rate_raw).unwrap();
        // let f = simple_soft_float::Float::from_bits_and_traits(rate_raw, props);

        let mut rate_buf = [0; 10]; // 1 bit sign, 15 bits exponent
        buf.read_exact(&mut rate_buf).unwrap();
        let sample_rate = parse_extended_precision_bytes(rate_buf);

        Ok(CommonChunk {
            size,
            num_channels,
            num_sample_frames,
            bit_rate,
            sample_rate,
        })
    }
}

pub struct SoundDataChunk {
    pub size: i32,
    pub offset: u32,
    pub block_size: u32,
    pub sound_data: Vec<u8>,
}

impl Chunk for SoundDataChunk {
    fn build<S: Read + Seek>(
        buf: Buffer<S>,
        id: ChunkID,
    ) -> Result<SoundDataChunk, ChunkError> {
        if &id != ids::SOUND {
            return Err(ChunkError::InvalidID(id));
        }

        let size = reader::read_i32_be(buf);
        let offset = reader::read_u32_be(buf);
        let block_size = reader::read_u32_be(buf);

        // TODO compare sound size with CommonChunk::num_sample_frames
        // size should be equal to num_sample_frames * num_channels.
        // According to the spec, `size` should account for offset + block_size + sound_data
        // or at least, it's implied? Either way, accounting for it causes current output
        // to make less sense.

        // let sound_size = size - 8; // offset + blocksize = 8 bytes
        // let start = buf.position() as usize;
        // let stop = start + sound_size as usize;
        let sound_size = size;

        let mut sound_data = vec![0; sound_size as usize];
        buf.read_exact(&mut sound_data).unwrap();

        Ok(SoundDataChunk {
            size,
            offset,
            block_size,
            sound_data,
        })
    }
}

pub struct Marker {
    id: i16,
    position: u32,
    marker_name: String,
}

pub struct MarkerChunk {
    pub size: i32,
    pub num_markers: u16,
    pub markers: Vec<Marker>,
}

pub enum TextChunkType {
    Name,
    Author,
    Copyright,
    Annotation,
}

pub struct TextChunk {
    pub chunk_type: TextChunkType,
    pub size: i32,
    pub text: String,
}

impl Chunk for TextChunk {
    fn build<S: Read + Seek>(
        buf: Buffer<S>,
        id: ids::ChunkID,
    ) -> Result<TextChunk, ChunkError> {
        let chunk_type = match &id {
            ids::NAME => TextChunkType::Name,
            ids::AUTHOR => TextChunkType::Author,
            ids::COPYRIGHT => TextChunkType::Copyright,
            ids::ANNOTATION => TextChunkType::Annotation,
            _ => return Err(ChunkError::InvalidID(id)),
        };

        let size = reader::read_i32_be(buf);
        let mut text_bytes = vec![0; size as usize];
        buf.read_exact(&mut text_bytes).unwrap();
        let text = String::from_utf8(text_bytes).unwrap();

        Ok(TextChunk {
            chunk_type,
            size,
            text,
        })
    }
}

pub struct ID3v2Chunk {
    version: [u8; 2],
}

// IMPORTANT - there is a COMM ID here as well. not a a problem
// if the id3 data is separated.
// v2.3 http://id3.org/id3v2.3.0
// v2.4 https://id3.org/id3v2.4.0-structure
impl Chunk for ID3v2Chunk {
    fn build<S: Read + Seek>(
        buf: Buffer<S>,
        id: ChunkID,
    ) -> Result<ID3v2Chunk, ChunkError> {
        if &id[0..3] != ids::ID3 && &id[1..] != ids::ID3 {
            return Err(ChunkError::InvalidID(id));
        }

        // TODO is this necessary? can we get this from id3 read
        let mut version = [0; 2];
        buf.seek(SeekFrom::Current(3)).unwrap();
        buf.read_exact(&mut version).unwrap();
        buf.seek(SeekFrom::Current(-5)).unwrap();

        // major versions up to 2.4, no minor versions known
        if version[0] > 4 || version[1] != 0 {
            return Err(ChunkError::InvalidID3Version(version));
        }

        // buffer MUST start with "ID3" or this call will fail
        let tag = id3::Tag::read_from(buf).unwrap();
        let frames: Vec<_> = tag.frames().collect();
        println!("id3 frames {:?}", frames);

        Ok(ID3v2Chunk { version })
    }
}
