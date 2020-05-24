extern crate bytes;
extern crate coreaudio_sys;
extern crate cpal;
extern crate id3;
extern crate log;
extern crate seek_bufread;

mod chunks;
mod extended;
mod ids;
pub mod reader;

mod test {
    use crate::reader::AiffReader;
    use coreaudio_sys::*;
    use cpal::{
        traits::{DeviceTrait, EventLoopTrait, HostTrait},
        Sample,
    };
    use std::fs::File;

    #[test]
    fn read() {
        // let args: Vec<String> = std::env::args().collect();
        // println!("args {:?}", args);
        let mut f = File::open("./devil.aiff").unwrap();
        // let mut f = File::open("./purp.aiff").unwrap();
        // let mut f = File::open(
        //     "/Volumes/jt-hd-osx/Music/LW Productions/Chemistry and Love/05 Phantom Creeps.aiff",
        // )
        // .unwrap();

        let mut reader = AiffReader::new(&mut f);
        reader.read().unwrap();
        let f = reader.form_chunk.as_ref().unwrap();
        let c = f.common().as_ref().unwrap();

        let host = cpal::default_host();
        let event_loop = host.event_loop();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let mut supported_formats_range = device
            .supported_output_formats()
            .expect("error while querying formats");

        let format = supported_formats_range
            .find(|f| c.sample_rate >= f.min_sample_rate.0 as f64)
            .unwrap()
            .with_max_sample_rate();
        let stream_id =
            event_loop.build_output_stream(&device, &format).unwrap();
        println!("{:?}", stream_id);
        event_loop
            .play_stream(stream_id)
            .expect("failed to play_stream");
        // TODO need to convert the values to signed integers first
        let mut samples: Vec<_> =
            reader.samples().iter().map(|s| s.to_f32()).collect();
        event_loop.run(move |_stream_id, _stream_result| {
            match _stream_result {
                Ok(cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut b),
                }) => {
                    let mut drain = samples.drain(..b.len());
                    for buf_spot in b.iter_mut() {
                        *buf_spot = drain.next().unwrap();
                    }
                }
                _ => println!("failed"),
            };
            // react to stream events and read or write stream data here
        });

        // for format in supported_formats_range {
        //     println!("fmt {:?}", format);
        // }
        // let format = supported_formats_range
        //     .next()
        //     .expect("no supported format?!")
        //     .with_max_sample_rate();

        // reader.samples();
    }
}
