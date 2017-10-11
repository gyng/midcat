extern crate cpal;
extern crate futures;
extern crate synthrs;
#[macro_use]
extern crate serde_derive;
extern crate docopt;

use std::thread;
use std::sync::Arc;

use docopt::Docopt;
use futures::stream::Stream;
use futures::task;
use futures::task::Executor;
use futures::task::Run;

use synthrs::synthesizer::{make_samples_from_midi, quantize};

const USAGE: &'static str = "
Play a MIDI file, ignoring instruments

Usage:
  midcat <file> [--volume=<frac>] [--speed=<times>]
  midcat (-h | --help)
  midcat --version

Options:
  -h --help                   Show this screen
  -v=<frac> --volume=<frac>   Play volume as a fraction (linear scale) [default: 1.0]
  -s=<times> --speed=<times>  Play speed as a fraction [default: 1.0]
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file: String,
    flag_volume: f64,
    flag_speed: f64,
}

struct MyExecutor;

impl Executor for MyExecutor {
    fn execute(&self, r: Run) {
        r.run();
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    assert!(args.flag_speed > 0.0);

    let endpoint = cpal::get_default_endpoint().unwrap();
    let format = endpoint
        .get_supported_formats_list()
        .unwrap()
        .next()
        .unwrap();

    let event_loop = cpal::EventLoop::new();
    let executor = Arc::new(MyExecutor);
    let (mut voice, stream) =
        cpal::Voice::new(&endpoint, &format, &event_loop).expect("failed to create voice/stream");

    let sample_rate = f64::from(format.samples_rate.0) / args.flag_speed;
    let source_samples = make_samples_from_midi(sample_rate as usize, &args.arg_file);
    let mut data_source = Arc::new(source_samples.into_iter());

    // This is for a simple waveform function
    // let mut data_source =
    //      synthrs::synthesizer::SamplesIter::new(
    //          sample_rate as u64,
    //          Box::new(synthrs::wave::SineWave(440.0))
    //     );

    task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
        let mut data_source = Arc::get_mut(&mut data_source).expect("failed to get arc");
        let channels = format.channels.len();

        match buffer {
            cpal::UnknownTypeBuffer::U16(mut buffer) => {
                let zipped = buffer.chunks_mut(channels).zip(&mut data_source);
                if zipped.size_hint() == (0, Some(0)) {
                    ::std::process::exit(0);
                }

                for (sample, value) in zipped {
                    for out in sample.iter_mut() {
                        *out = quantize::<u16>(args.flag_volume * value);
                    }
                }
            }
            cpal::UnknownTypeBuffer::I16(mut buffer) => {
                let zipped = buffer.chunks_mut(channels).zip(&mut data_source);
                if zipped.size_hint() == (0, Some(0)) {
                    ::std::process::exit(0);
                }

                for (sample, value) in zipped {
                    for out in sample.iter_mut() {
                        *out = quantize::<i16>(args.flag_volume * value);
                    }
                }
            }
            cpal::UnknownTypeBuffer::F32(mut buffer) => {
                let zipped = buffer.chunks_mut(channels).zip(&mut data_source);
                if zipped.size_hint() == (0, Some(0)) {
                    ::std::process::exit(0);
                }

                for (sample, value) in zipped {
                    for out in sample.iter_mut() {
                        *out = (args.flag_volume * value) as f32;
                    }
                }
            }
        }
        Ok(())
    })).execute(executor);

    thread::spawn(move || { voice.play(); });

    event_loop.run();
}
