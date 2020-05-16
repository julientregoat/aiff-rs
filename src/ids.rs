use super::reader::Buffer;
use std::io::{Read, Seek, SeekFrom};

pub type ChunkID = [u8; 4];

pub const FORM: &ChunkID = b"FORM";
pub const AIFF: &ChunkID = b"AIFF";
pub const COMMON: &ChunkID = b"COMM";
pub const SOUND: &ChunkID = b"SSND";
pub const MARKER: &ChunkID = b"MARK";
pub const INSTRUMENT: &ChunkID = b"INST";
pub const MIDI: &ChunkID = b"MIDI";
pub const RECORDING: &ChunkID = b"AESD";
pub const APPLICATION: &ChunkID = b"APPL";
pub const COMMENT: &ChunkID = b"COMT";
pub const NAME: &ChunkID = b"NAME";
pub const AUTHOR: &ChunkID = b"AUTH";
pub const COPYRIGHT: &ChunkID = b"(c) ";
pub const ANNOTATION: &ChunkID = b"ANNO";

pub const AIFF_C: &ChunkID = b"AIFC";
pub const FVER: &ChunkID = b"FVER"; // 'Format version' - for AIFF C

pub const CHAN: &ChunkID = b"CHAN";
pub const BASC: &ChunkID = b"basc";
pub const TRNS: &ChunkID = b"trns";
pub const CATE: &ChunkID = b"cate";

pub const TAG: &[u8; 3] = b"TAG";
pub const ID3: &[u8; 3] = b"ID3";
