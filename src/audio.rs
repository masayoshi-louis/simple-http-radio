use std::cmp::{max, min};

use cpal::{Device, Format};
use failure::Error;
use failure::format_err;
use futures::Stream;
use futures::sync::mpsc::UnboundedSender;

use lazy_static::lazy_static;

use crate::flac;

lazy_static! {
static ref DEVICE: Device = cpal::default_input_device().expect("no default device");
static ref FORMAT: Format = DEVICE.default_input_format().expect("no default format");
}

#[inline]
pub fn sample_rate() -> u32 {
    format().sample_rate.0
}

#[inline]
pub fn bit_depth() -> usize {
    format().data_type.sample_size() * 8
}

#[inline]
pub fn channels() -> u16 {
    format().channels
}

pub fn print_device_info() {
    info!("Device: {}", device().name());
    info!("Format: {:?}", format());
}

pub fn start() -> impl Stream<Item=Vec<u8>, Error=Error> {
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_input_stream(device(), format())
        .expect("Failed to build input stream");
    event_loop.play_stream(stream_id);
    let (tx, rx) = futures::sync::mpsc::unbounded();
    std::thread::spawn(move || {
        let mut active = true;

        let bps = min(24, bit_depth() as u32);
        info!("Creating new session. SampleRate={}, bps={}, channels={}.", sample_rate(), bps, channels());

        let mut encoder = flac::StreamEncoder::create();
        encoder.set_bits_per_sample(bps);
        encoder.set_sample_rate(sample_rate());
        encoder.set_channels(channels() as u32);
        encoder.set_compression_level(5);
        encoder.set_verify(true);

        let mut cb = |buffer: &[u8], _: usize, _: usize| {
            if !send(buffer.to_owned(), &tx) {
                info!("Session stopped");
                active = false;
            }
            Ok(())
        };

        encoder.init_ogg_stream_non_seekable(&mut cb);

        if !encoder.is_ok() {
            error!("FLAC Encoder state: {}", encoder.get_state());
            panic!("failed to initialize FLAC encoder")
        }

        info!("EventLoop thread started");
        let mut vec: Vec<i32> = Vec::with_capacity(1024);
        let mut samples: usize = 0;
        event_loop.run(|stream_id, data| {
            if !active {
                event_loop.destroy_stream(stream_id);
                return;
            }
            vec.clear();
            match data {
                cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::U16(buffer) } => {
                    samples = buffer.len();
                    debug!("processing {} U16 samples", samples);
                    for sample in buffer.iter() {
                        let sample = cpal::Sample::to_i16(sample);
                        vec.push(sample as i32);
                    }
                }
                cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::I16(buffer) } => {
                    samples = buffer.len();
                    debug!("processing {} I16 samples", samples);
                    for &sample in buffer.iter() {
                        vec.push(sample as i32);
                    }
                }
                cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::F32(buffer) } => {
                    samples = buffer.len();
                    debug!("processing {} F32 samples", samples);
                    for &sample in buffer.iter() {
                        let mut int_sample = (sample * 8388608.0f32).round() as i32;
                        int_sample = max(int_sample, -8388608);
                        int_sample = min(int_sample, 8388607);
                        vec.push(int_sample);
                    }
                }
                _ => {
                    samples = 0;
                }
            }

            encoder.process_interleaved(&vec, samples);
            if !encoder.is_ok() {
                error!("FLAC Encoder state: {}", encoder.get_state());
                panic!("FLAC encoder failure")
            }
        });
        info!("EventLoop thread quit");
    });
    rx.map_err(|_| format_err!("Error"))
}

#[inline]
fn send(buff: Vec<u8>, tx: &UnboundedSender<Vec<u8>>) -> bool {
    let size = buff.len();
    let successful = tx.unbounded_send(buff).is_ok();
    if successful {
        debug!("transferred {} bytes", size);
    }
    return successful;
}

#[inline]
fn device() -> &'static Device {
    &DEVICE
}

#[inline]
fn format() -> &'static Format {
    &FORMAT
}
