use super::{
    chunks::{self, Chunk, FormChunk},
    ids,
    samples::SampleType,
};
use seek_bufread::BufReader;
use std::io::{Read, Seek, SeekFrom};

pub type Buffer<'a, Source> = &'a mut BufReader<Source>;

// TODO samples iterator, enable seeking by duration fn
// TODO diffeerent types of reader structs?
// AiffAudioReader / AiffCompleteReader (id3 optional)
pub struct AiffReader<Source> {
    buf: BufReader<Source>,
    form_chunk: Option<FormChunk>,
    // pub id3v1_tags: Vec<chunks::ID3v1Chunk>, // should this be optional? or separate
    id3v2_tags: Vec<chunks::ID3v2Chunk>, // should this be optional? or separate
}

impl<Source: Read + Seek> AiffReader<Source> {
    pub fn new(s: Source) -> AiffReader<Source> {
        AiffReader {
            buf: BufReader::new(s),
            form_chunk: None,
            id3v2_tags: vec![],
            // id3v1_tags: vec![],
        }
    }

    pub fn read(&mut self) -> Result<(), chunks::ChunkError> {
        let form_id = read_chunk_id(&mut self.buf);
        let mut form = FormChunk::parse(&mut self.buf, form_id)?;

        while self.buf.available() >= 4 {
            let id = read_chunk_id(&mut self.buf);

            // once the common and form are detected, we can loop
            // buffer position is right past the id
            match &id {
                ids::COMMON => {
                    println!("Common chunk detected");
                    let common =
                        chunks::CommonChunk::parse(&mut self.buf, id).unwrap();
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
                    let sound =
                        chunks::SoundDataChunk::parse(&mut self.buf, id)
                            .unwrap();
                    println!(
                        "SOUND chunk detected size {} offset {} block size {}",
                        sound.size, sound.offset, sound.block_size
                    );
                    form.set_sound(sound);
                }
                ids::MARKER => {
                    let mark =
                        chunks::MarkerChunk::parse(&mut self.buf, id).unwrap();
                    println!("MARKER chunk detected {:?}", mark);
                    form.add_marker_chunk(mark);
                }
                ids::INSTRUMENT => {
                    let inst =
                        chunks::InstrumentChunk::parse(&mut self.buf, id)
                            .unwrap();
                    println!("INSTRUMENT chunk detected {:?}", inst);
                    form.set_instrument(inst);
                }
                ids::MIDI => {
                    let midi = chunks::MIDIDataChunk::parse(&mut self.buf, id)
                        .unwrap();
                    println!("MIDI chunk detected {:?}", midi);
                    form.add_midi_chunk(midi);
                }
                ids::RECORDING => {
                    let rec =
                        chunks::AudioRecordingChunk::parse(&mut self.buf, id)
                            .unwrap();
                    println!("RECORDING chunk detected {:?}", rec);
                    form.set_recording(rec);
                }
                ids::APPLICATION => {
                    let app = chunks::ApplicationSpecificChunk::parse(
                        &mut self.buf,
                        id,
                    )
                    .unwrap();
                    println!("APPLICATION chunk detected {:?}", app);
                    form.add_app_chunk(app);
                }
                ids::COMMENTS => {
                    let comm = chunks::CommentsChunk::parse(&mut self.buf, id)
                        .unwrap();
                    println!("COMMENT chunk detected {:?}", comm);
                    form.set_comments(comm);
                }
                ids::NAME | ids::AUTHOR | ids::COPYRIGHT | ids::ANNOTATION => {
                    let text =
                        chunks::TextChunk::parse(&mut self.buf, id).unwrap();
                    println!("TEXT chunk detected: {}", text.text);
                    form.add_text_chunk(text);
                }
                ids::FVER => {
                    unimplemented!("FVER chunk detected");
                }
                // 3 bytes "ID3" identifier
                // TODO merge both options
                // ID3 chunks aren't stored in the FORM chunk. should they
                // be stored next to the form chunk in the reader?
                [73, 68, 51, _] => {
                    self.buf.seek(SeekFrom::Current(-4)).unwrap();
                    match chunks::ID3v2Chunk::parse(&mut self.buf, id) {
                        Ok(chunk) => self.id3v2_tags.push(chunk),
                        Err(e) => {
                            println!("Build ID3 chunk failed {:?}", e);
                            self.buf.seek(SeekFrom::Current(3)).unwrap();
                        }
                    }
                }
                [_, 73, 68, 51] => {
                    self.buf.seek(SeekFrom::Current(-3)).unwrap();
                    match chunks::ID3v2Chunk::parse(&mut self.buf, id) {
                        Ok(chunk) => self.id3v2_tags.push(chunk),
                        Err(e) => {
                            println!("Build ID3 chunk failed {:?}", e);
                            self.buf.seek(SeekFrom::Current(3)).unwrap();
                        }
                    }
                }
                [84, 65, 71, _] => println!("v1 id3"), // "TAG_"
                [_, 84, 65, 71] => println!("v1 id3"), // "_TAG"
                ids::CHAN | ids::BASC | ids::TRNS | ids::CATE => {
                    unimplemented!("apple stuff detected")
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
        // set position to end?

        Ok(())
    }

    pub fn form(&self) -> &Option<FormChunk> {
        &self.form_chunk
    }

    // TODO need to check available
    // TODO return result iterator or complete buffer of data
    // TODO pack frams
    // should return a generic AiffSample<u8/u16/u32> etc
    // TODO samples is most likely integers

    pub fn samples<T: SampleType>(&self) -> Vec<T> {
        let f = self.form_chunk.as_ref().unwrap();
        let s = f.sound().as_ref().unwrap();
        let c = f.common().as_ref().unwrap();

        // a sample point is the sound data for a single channel of audio
        // sample points containn <bit_rate> bits of data
        // a sample frame contains sample points for all channels
        // playback occurs at <sample_rate> frames per second
        // num samples is always > 0 so shouldn't be any conversion issues
        // maybe it should be stored as a u16?
        let sample_points =
            (c.num_sample_frames * c.num_channels as u32) as usize;
        println!("sample points {:?}", sample_points);

        let mut samples = Vec::with_capacity(sample_points);
        let mut bytes_per_point = (c.bit_rate / 8) as usize;
        if c.bit_rate % 8 != 0 {
            bytes_per_point += 1;
        }

        for point in 0..sample_points {
            samples.push(T::parse(&s.sound_data, point * bytes_per_point, c.bit_rate));
        }

        samples
    }

    // TODO create samples iterator for better performance
}

// enums are always the max possible size, so neeeds to be structs and traits

// TODO remove panics
// TODO move these into their own file - what's a good name?

pub fn read_chunk_id(r: &mut impl Read) -> ids::ChunkID {
    let mut id = [0; 4];
    if let Err(e) = r.read_exact(&mut id) {
        panic!("unable to read_u8 {:?}", e)
    }
    id
}

pub fn read_u8(r: &mut impl Read) -> u8 {
    let mut b = [0; 1];
    if let Err(e) = r.read_exact(&mut b) {
        panic!("unable to read_u8 {:?}", e)
    }
    b[0]
}

pub fn read_u16_be(r: &mut impl Read) -> u16 {
    let mut b = [0; 2];
    if let Err(e) = r.read_exact(&mut b) {
        panic!("unable to read_u8 {:?}", e)
    }
    u16::from_be_bytes(b)
}

pub fn read_u32_be(r: &mut impl Read) -> u32 {
    let mut b = [0; 4];
    if let Err(e) = r.read_exact(&mut b) {
        panic!("unable to read_i32_be {:?}", e)
    }
    u32::from_be_bytes(b)
}

pub fn read_i8_be(r: &mut impl Read) -> i8 {
    let mut b = [0; 1];
    if let Err(e) = r.read_exact(&mut b) {
        panic!("unable to read_i32_be {:?}", e)
    }
    i8::from_be_bytes(b)
}

pub fn read_i16_be(r: &mut impl Read) -> i16 {
    let mut b = [0; 2];
    if let Err(e) = r.read_exact(&mut b) {
        panic!("unable to read_i32_be {:?}", e)
    }
    i16::from_be_bytes(b)
}

pub fn read_i32_be(r: &mut impl Read) -> i32 {
    let mut b = [0; 4];
    if let Err(e) = r.read_exact(&mut b) {
        panic!("unable to read_i32_be {:?}", e)
    }
    i32::from_be_bytes(b)
}

// TODO testme with pascal strings
pub fn read_pstring<R: Read + Seek>(r: &mut R) -> String {
    let len = read_u8(r);
    let mut str_buf = vec![0; len as usize];
    r.read_exact(&mut str_buf).unwrap();

    if len % 2 > 0 {
        // skip pad byte if odd
        r.seek(SeekFrom::Current(1)).unwrap();
    }

    String::from_utf8(str_buf).unwrap()
}
