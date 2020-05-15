extern crate bytes;
//extern crate cpal;
extern crate rust_decimal;

mod chunks;
mod ids;
pub mod reader;

mod test {
    use crate::reader::AiffReader;
    use std::fs::File;

    #[test]
    fn read() {
        let mut f = File::open("./purp.aiff").unwrap();

        let mut reader = AiffReader::new(&mut f);
        reader.read();
    }
}
