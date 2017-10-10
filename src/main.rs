extern crate cpal;
extern crate futures;
extern crate synthrs;

use std::thread;
use std::sync::{Arc, Mutex};
use std::ops::Deref;

use futures::stream::Stream;
use futures::task;
use futures::task::Executor;
use futures::task::Run;

use synthrs::synthesizer::{SamplesIter, quantize, make_samples_from_midi};
use synthrs::wave::SineWave;

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

    // This works
    let mut data_source = SamplesIter::new(format.samples_rate.0 as u64, Box::new(SineWave(440.0)));

    // let mut data_source = make_samples_from_midi(44100, "mountainking.mid").iter().cloned();
    // let mut data_source = Arc::new(make_samples_from_midi(44100, "mountainking.mid").iter().cloned());

    task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
        // let mut data_source = Arc::get_mut(&mut data_source).unwrap();
        match buffer {
            cpal::UnknownTypeBuffer::U16(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(
                    &mut data_source,
                )
                {
                    for out in sample.iter_mut() {
                        *out = 0 // quantize::<u16>(value); // might be really loud
                    }
                }
            }
            cpal::UnknownTypeBuffer::I16(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(
                    &mut data_source,
                )
                {
                    for out in sample.iter_mut() {
                        *out = 0 // quantize::<i16>(value); // might be really loud
                    }
                }
            }
            cpal::UnknownTypeBuffer::F32(mut buffer) => {
                for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(
                    &mut data_source,
                )
                {
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
