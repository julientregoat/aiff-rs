use super::extended::parse_extended_precision_bytes;
use super::{
    ids::{self, ChunkID},
    reader::{self, Buffer},
};
use id3;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Div;

#[derive(Debug)]
pub enum ChunkError {
    InvalidID(ChunkID),
    InvalidFormType(ChunkID),
    InvalidID3Version([u8; 2]),
    InvalidSize(i32, i32), // expected, got
}

// TODO rename 'build'
pub trait Chunk {
    fn build(
        buffer: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<Self, ChunkError>
    where
        Self: Sized;
}

// TODO samples iterator, enable seeking by duration fn
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

    pub fn duration(&self) -> Option<f64> {
        if let Some(common) = &self.common {
            Some((common.num_sample_frames as f64).div(common.sample_rate))
        } else {
            None
        }
    }
}

impl Chunk for FormChunk {
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<FormChunk, ChunkError> {
        if &id != ids::FORM {
            return Err(ChunkError::InvalidID(id));
        }

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
    fn build(
        buf: Buffer<impl Read + Seek>,
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
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<SoundDataChunk, ChunkError> {
        if &id != ids::SOUND {
            return Err(ChunkError::InvalidID(id));
        }

        let size = reader::read_i32_be(buf);
        let offset = reader::read_u32_be(buf);
        let block_size = reader::read_u32_be(buf);

        let sound_size = size - 8; // account for offset + block size bytes
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

type MarkerId = i16;
pub struct Marker {
    id: MarkerId,
    position: u32,
    marker_name: String,
}

impl Marker {
    // TODO return result
    pub fn from_reader<R: Read + Seek>(r: &mut R) -> Marker {
        let id = reader::read_i16_be(r);
        let position = reader::read_u32_be(r);
        let marker_name = reader::read_pstring(r);

        Marker {
            id,
            position,
            marker_name,
        }
    }
}

pub struct MarkerChunk {
    pub size: i32,
    pub num_markers: u16,
    pub markers: Vec<Marker>,
}

impl Chunk for MarkerChunk {
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<MarkerChunk, ChunkError> {
        if &id != ids::MARKER {
            return Err(ChunkError::InvalidID(id));
        }

        let size = reader::read_i32_be(buf);
        let num_markers = reader::read_u16_be(buf);
        let mut markers = Vec::with_capacity(num_markers as usize);
        // is it worth it to read all markers at once ant create from buf?
        // or does the usage of BufReader make it irrelevant?
        for _ in 0..num_markers {
            markers.push(Marker::from_reader(buf));
        }

        Ok(MarkerChunk {
            size,
            num_markers,
            markers,
        })
    }
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
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
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

        if size % 2 > 0 {
            // if odd, pad byte present - skip it
            buf.seek(SeekFrom::Current(1)).unwrap();
        }

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

// should this be an optional feature? maybe consumer already has id3 parsing
impl Chunk for ID3v2Chunk {
    fn build(
        buf: Buffer<impl Read + Seek>,
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

pub struct Loop {
    // 0 no looping / 1 foward loop / 2 forward backward loop - use enum?
    play_mode: i16,
    begin_loop: MarkerId,
    end_loop: MarkerId,
}

impl Loop {
    // TODO return result
    pub fn from_reader(r: &mut impl Read) -> Loop {
        let play_mode = reader::read_i16_be(r);
        let begin_loop = reader::read_i16_be(r);
        let end_loop = reader::read_i16_be(r);

        Loop {
            play_mode,
            begin_loop,
            end_loop,
        }
    }
}

// midi note value range = 0..127 (? not the full range?)
pub struct InstrumentChunk {
    size: i32,
    base_note: i8,     // MIDI
    detune: i8,        // -50..50
    low_note: i8,      // MIDI
    high_note: i8,     // MIDI
    low_velocity: i8,  // MIDI
    high_velocity: i8, // MIDI
    gain: i16,         // in db
    sustain_loop: Loop,
    release_loop: Loop,
}

impl Chunk for InstrumentChunk {
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<InstrumentChunk, ChunkError> {
        if &id != ids::INSTRUMENT {
            return Err(ChunkError::InvalidID(id));
        }

        let size = reader::read_i32_be(buf);
        let base_note = reader::read_i8_be(buf);
        let detune = reader::read_i8_be(buf);
        let low_note = reader::read_i8_be(buf);
        let high_note = reader::read_i8_be(buf);
        let low_velocity = reader::read_i8_be(buf);
        let high_velocity = reader::read_i8_be(buf);
        let gain = reader::read_i16_be(buf);

        let sustain_loop = Loop::from_reader(buf);
        let release_loop = Loop::from_reader(buf);

        Ok(InstrumentChunk {
            size,
            base_note,
            detune,
            low_note,
            high_note,
            low_velocity,
            high_velocity,
            gain,
            sustain_loop,
            release_loop,
        })
    }
}

pub struct MIDIDataChunk {
    size: i32,
    data: Vec<u8>,
}

impl Chunk for MIDIDataChunk {
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<MIDIDataChunk, ChunkError> {
        if &id != ids::MIDI {
            return Err(ChunkError::InvalidID(id));
        }

        let size = reader::read_i32_be(buf);

        let mut data = vec![0; size as usize];
        buf.read_exact(&mut data).unwrap();

        Ok(MIDIDataChunk { size, data })
    }
}

pub struct AudioRecordingChunk {
    size: i32,
    // AESChannelStatusData
    // specified in "AES Recommended Practice for Digital Audio Engineering"
    data: [u8; 24],
}

impl Chunk for AudioRecordingChunk {
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<AudioRecordingChunk, ChunkError> {
        if &id != ids::RECORDING {
            return Err(ChunkError::InvalidID(id));
        }

        let size = reader::read_i32_be(buf);
        if size != 24 {
            return Err(ChunkError::InvalidSize(24, size));
        }

        let mut data = [0; 24];
        buf.read_exact(&mut data).unwrap();

        Ok(AudioRecordingChunk { size, data })
    }
}

pub struct ApplicationSpecificChunk {
    size: i32,
    application_signature: ChunkID, // TODO check if bytes should be i8
    data: Vec<i8>,
}

impl Chunk for ApplicationSpecificChunk {
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<ApplicationSpecificChunk, ChunkError> {
        if &id != ids::APPLICATION {
            return Err(ChunkError::InvalidID(id));
        }

        let size = reader::read_i32_be(buf);
        let application_signature = reader::read_chunk_id(buf); // TODO verify
        let mut data = vec![0; (size - 4) as usize]; // account for sig size
        buf.read_exact(&mut data).unwrap();

        Ok(ApplicationSpecificChunk {
            size,
            application_signature,
            data: data.iter().map(|byte| i8::from_be_bytes([*byte])).collect(),
        })
    }
}

pub struct Comment {
    timestamp: u32,
    marker_id: MarkerId,
    count: u16,
    text: String, // padded to an even # of bytes
}

impl Comment {
    // TODO return result
    pub fn from_reader(r: &mut impl Read) -> Comment {
        let timestamp = reader::read_u32_be(r);
        let marker_id = reader::read_i16_be(r);
        let count = reader::read_u16_be(r);

        let mut str_buf = vec![0; count as usize];
        r.read_exact(&mut str_buf).unwrap();
        let text = String::from_utf8(str_buf).unwrap();

        Comment {
            timestamp,
            marker_id,
            count,
            text,
        }
    }
}

pub struct CommentsChunk {
    size: i32,
    num_comments: u16,
    comments: Vec<Comment>,
}

impl Chunk for CommentsChunk {
    fn build(
        buf: Buffer<impl Read + Seek>,
        id: ChunkID,
    ) -> Result<CommentsChunk, ChunkError> {
        if &id != ids::COMMENTS {
            return Err(ChunkError::InvalidID(id));
        }

        let size = reader::read_i32_be(buf);
        let num_comments = reader::read_u16_be(buf);

        let mut comments = Vec::with_capacity(num_comments as usize);
        for _ in 0..num_comments {
            comments.push(Comment::from_reader(buf))
        }

        Ok(CommentsChunk {
            size,
            num_comments,
            comments,
        })
    }
}
