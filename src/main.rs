extern crate cpal;
extern crate futures;
extern crate synthrs;

use std::thread;
use std::sync::Arc;

use futures::stream::Stream;
use futures::task;
use futures::task::Executor;
use futures::task::Run;

use synthrs::synthesizer::make_samples_from_midi;

struct MyExecutor;

impl Executor for MyExecutor {
    fn execute(&self, r: Run) {
        r.run();
    }
}

fn main() {
    let endpoint = cpal::get_default_endpoint().unwrap();
    let format = endpoint
        .get_supported_formats_list()
        .unwrap()
        .next()
        .unwrap();
    let event_loop = cpal::EventLoop::new();
    let executor = Arc::new(MyExecutor);

    let (mut voice, stream) = cpal::Voice::new(&endpoint, &format, &event_loop).unwrap();

    let source_samples = make_samples_from_midi(format.samples_rate.0 as usize, "mountainking.mid");
    let mut data_source = Arc::new(source_samples.into_iter());

    task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
        let mut data_source = Arc::get_mut(&mut data_source).unwrap();
        let channels = format.channels.len();

        match buffer {
            cpal::UnknownTypeBuffer::U16(mut _buffer) => {
                unimplemented!();
            }
            cpal::UnknownTypeBuffer::I16(mut _buffer) => {
                // `quantize` can be used to convert the f64 to i16/u16, but
                // since it cannot be tested on my machines it is left unimplemented
                // quantize::<i16>(value); // might be really loud
                unimplemented!();
            }
            cpal::UnknownTypeBuffer::F32(mut buffer) => {
                let zipped = buffer.chunks_mut(channels).zip(&mut data_source);
                if zipped.size_hint() == (0, Some(0)) {
                    ::std::process::exit(0);
                }

                for (sample, value) in zipped {
                    for out in sample.iter_mut() {
                        *out = value as f32;
                    }
                }
            }
        }
        Ok(())
    })).execute(executor);

    thread::spawn(move || {
        voice.play();
    });

    event_loop.run();
}
