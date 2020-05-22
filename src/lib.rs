extern crate bytes;
extern crate id3;
extern crate log;
extern crate seek_bufread;

mod chunks;
mod extended;
mod ids;
pub mod reader;

mod test {
    // extern crate cpal;
    use crate::reader::AiffReader;
    use std::fs::File;

    #[test]
    fn read() {
        let args: Vec<String> = std::env::args().collect();
        println!("args {:?}", args);
        let mut f = File::open("./devil.aiff").unwrap();
        // let mut f = File::open("./purp.aiff").unwrap();

        let mut reader = AiffReader::new(&mut f);
        reader.read().unwrap();
    }
}
