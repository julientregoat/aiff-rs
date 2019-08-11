pub type ID = [u8; 4];

pub const FORM: &ID = b"FORM";
pub const AIFF: &ID = b"AIFF";
pub const COMMON: &ID = b"COMM";
pub const SOUND: &ID = b"SSND";
pub const MARKER: &ID = b"MARK";
pub const INSTRUMENT: &ID = b"INST";
pub const MIDI: &ID = b"MIDI";
pub const RECORDING: &ID = b"AESD";
pub const APPLICATION: &ID = b"APPL";
pub const COMMENT: &ID = b"COMT";
pub const NAME: &ID = b"NAME";
pub const AUTHOR: &ID = b"AUTH";
pub const COPYRIGHT: &ID = b"(c) ";
pub const ANNOTATION: &ID = b"ANNO";

pub const AIFF_C: &ID = b"AIFC";
pub const FVER: &ID = b"FVER"; // 'Format version' - for AIFF C

pub const CHAN: &ID = b"CHAN";
pub const BASC: &ID = b"basc";
pub const TRNS: &ID = b"trns";
pub const CATE: &ID = b"cate";

pub const ID3: &ID = b"ID3 "; // should only be 3 bytes - should this be used?
