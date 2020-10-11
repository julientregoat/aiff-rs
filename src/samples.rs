pub struct AiffSamples {
    pos: usize,
}

pub trait SampleType: Sized {
    fn parse(data: &[u8], pos: usize, bit_width: i16) -> Self;
}

// TODO handle padding for non standard bit rates
// TODO handle offset + blocksize parameters
// FIXME proper error handling

impl SampleType for i8 {
    fn parse(data: &[u8], pos: usize, bit_width: i16) -> Self {
        if bit_width < 8 {
            unimplemented!("only 16 24 32 bit supported, got {:?}", bit_width)
        } else if bit_width > 8 {
            panic!("invalid bit width supplied. expected 8 vs {:?}", bit_width)
        }
        i8::from_be_bytes([data[pos]])
    }
}

impl SampleType for i16 {
    fn parse(data: &[u8], pos: usize, bit_width: i16) -> Self {
        if bit_width < 16 {
            unimplemented!("only 16 24 32 bit supported, got {:?}", bit_width)
        } else if bit_width > 16 {
            panic!("invalid bit width supplied. expected 16 vs {:?}", bit_width)
        }
        i16::from_be_bytes([data[pos], data[pos + 1]])
    }
}

impl SampleType for i32 {
    fn parse(data: &[u8], pos: usize, bit_width: i16) -> Self {
        match bit_width {
            32 => i32::from_be_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
            ]),
            24 => {
                i32::from_be_bytes([0x0, data[pos], data[pos + 1], data[pos + 2]])
            }
            b if b <= 16 => panic!("invalid bit width supplied. expected 24 or 32 vs {:?}", b),
            b => unimplemented!("only 16 24 32 bit supported, got {:?}", b),
        }
    }
}

// impl SampleType for f32 {
//     fn parse(data: &[u8], pos: usize, bit_width: i16) -> Self {
//         let int_val = i32::parse(data, pos, bit_width);
//         int_val as f32
//     }
// }

// I made this before deciding not to implement an iterator. so this will
// probably need to be refactored when iterator is implemented

pub struct Samples8 {
    point: usize,
    sound_data: Vec<u8>,
}

impl Iterator for Samples8 {
    type Item = i8;

    fn next(&mut self) -> Option<Self::Item> {
        let target = self.point * 4;
        if (target + 2) > self.sound_data.len() {
            // out of bounds, panic?
            return None;
        }

        self.point += 1;

        Some(i8::from_be_bytes([self.sound_data[target]]))
    }
}

pub struct Samples16 {
    point: usize,
    sound_data: Vec<u8>,
}

impl Iterator for Samples16 {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let target = self.point * 4;
        if (target + 2) > self.sound_data.len() {
            // out of bounds, panic?
            return None;
        }

        self.point += 1;

        Some(i16::from_be_bytes([
            self.sound_data[target],
            self.sound_data[target + 1],
        ]))
    }
}

pub struct Samples32 {
    point: usize,
    sound_data: Vec<u8>,
}

// generic iterator possible?
impl Samples32 {
    pub fn new(sound_data: Vec<u8>) -> Self {
        // TODO should uneven size (relative to points or frames) panic?
        Samples32 {
            point: 0,
            sound_data,
        }
    }
}

impl Iterator for Samples32 {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let target = self.point * 4;
        if (target + 3) > self.sound_data.len() {
            // out of bounds, panic?
            return None;
        }

        self.point += 1;

        Some(i32::from_be_bytes([
            self.sound_data[target],
            self.sound_data[target + 1],
            self.sound_data[target + 2],
            self.sound_data[target + 3],
        ]))
    }
}
