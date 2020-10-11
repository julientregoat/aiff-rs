extern crate aiff;
extern crate cpal;

use aiff::reader::AiffReader;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleRate,
};
use std::fs::File;

fn main() {
    // let file = File::open("./devil.aiff").unwrap();
    // let file = File::open("./down.aiff").unwrap();
    let file = File::open("./kc2496.aiff").unwrap();
    // let file = File::open("./kc3296.aiff").unwrap();
    // .unwrap();

    let mut reader = AiffReader::new(file);
    reader.read().unwrap();
    let comm = reader.form().as_ref().unwrap().common().as_ref().unwrap();

    let host = cpal::default_host();

    host.devices().unwrap().for_each(|d| {
        println!("device {:?}", d.name());

        match d.supported_output_configs() {
            Ok(confs) => {
                confs.for_each(|f| {
                    println!(
                        "format chan {:?} min {:?} max {:?} format {:?}",
                        f.channels(),
                        f.min_sample_rate(),
                        f.max_sample_rate(),
                        f.sample_format()
                    )
                });
            }
            Err(e) => println!("Failed to get output configs {:?}", e),
        }
    });

    let device = host
        .default_output_device()
        .expect("no output device available");

    println!("selected device {:?}", device.name());

    let mut supported_formats_range = device
        .supported_output_configs()
        .expect("error while querying formats");

    let stream_config = supported_formats_range
        .find(|f| {
            let min_float = f.min_sample_rate().0 as f64;
            let max_float = f.max_sample_rate().0 as f64;
            comm.sample_rate >= min_float && comm.sample_rate <= max_float
        })
        .unwrap()
        .with_sample_rate(SampleRate(comm.sample_rate as u32))
        .config();

    println!("stream config {:?}", stream_config);

    let samples: Vec<_> =
        reader.samples::<i32>().iter().map(|s| s.to_f32()).collect();

    // let samples: Vec<_> = reader.samples().iter().map(|s| s.to_f32()).collect();
    let mut idx = 0;

    let duration = comm.num_sample_frames as f64 / comm.sample_rate;
    println!("samples collected duration {:?}", duration);

    let nchan = comm.num_channels as u16;
    let pad = stream_config.channels - nchan;

    let stream = device
        .build_output_stream(
            &stream_config,
            move |data: &mut [f32], conf: &cpal::OutputCallbackInfo| {
                println!(
                    "callback {:?} {:?} {:?}",
                    idx,
                    data.len(),
                    conf.timestamp()
                );

                for frame in data.chunks_mut((nchan + pad) as usize) {
                    for point in 0..nchan as usize {
                        frame[point] = samples[idx];
                        idx += 1;
                    }
                }
            },
            move |err| {
                println!("ERROR");
                panic!("stream error {:?}", err);
                // react to errors here.
            },
        )
        .unwrap();

    println!("built stream");

    stream.play().unwrap();

    println!("playing");

    std::thread::sleep_ms((duration * 1000.0).trunc() as u32);

    println!("slept");
}
