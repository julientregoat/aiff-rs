//extern crate cpal;
extern crate bytes;
extern crate id3;
extern crate seek_bufread;

mod chunks;
mod ids;
pub mod reader;

mod test {
    use crate::reader::AiffReader;
    use std::fs::File;

    #[test]
    fn read() {
        let args: Vec<String> = std::env::args().collect();
        println!("args {:?}", args);
        // let mut f = File::open("./purp.aiff").unwrap();
        // let mut f = File::open("/Volumes/jt-hd-osx/Music/Kode9/Tempa Allstars Vol. 2/03 Babylon (Dub Mix).aiff").unwrap();
        let mut f = File::open("./purp.aiff").unwrap();

        let mut reader = AiffReader::new(&mut f);
        reader.read().unwrap();
    }
}
