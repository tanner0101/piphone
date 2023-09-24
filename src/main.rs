mod net_util;
mod poormans;
mod gpio_util;
mod rodio_util;

use std;
use std::io;
use biquad::ToHertz;
use rodio::{cpal, DeviceTrait};
use rodio::cpal::traits::{HostTrait, StreamTrait};

fn main() -> io::Result<()> {
    let ctx = net_util::Context::new("72.180.248.254");
    let (_device, mic_config, _stream) = start_microphone(&ctx);
    start_speaker(&ctx, mic_config);
    return io::Result::Ok(());
}

fn start_microphone(ctx: &net_util::Context) -> (rodio::Device, rodio::SupportedStreamConfig, cpal::Stream) {
    println!("\nstarting microphone...");
    let host = cpal::default_host();
    let device = rodio_util::find_device_by_name(
        host.input_devices().unwrap(),
        "sysdefault:CARD=X",
    ).unwrap();
    let config = device.default_input_config().unwrap();
    let mut filter = poormans::Filter::new(config.sample_rate().0.hz());
    let send_socket = ctx.send_socket.try_clone().unwrap();
    let remote_addr = ctx.remote_addr.clone();
    let input_stream = device.build_input_stream(
        &config.clone().into(),
        move |_data: &[i16], _: &cpal::InputCallbackInfo| {
            if !gpio_util::is_call_active() {
                return;
            }
            let len = _data.len() * std::mem::size_of::<i16>();
            let data: Vec<i16> = _data.iter().map(|x| filter.run(*x)).collect();
            let ptr = data.as_ptr() as *const u8;
            let slice_u8: &[u8] = unsafe { std::slice::from_raw_parts(ptr, len) };
            send_socket.send_to(slice_u8, &remote_addr).expect("failed to send mic data");
        },
        move |err| {
            eprintln!("an error occurred on stream: {}", err);
        },
        Option::None,
    )
        .unwrap();

    input_stream
        .play()
        .unwrap();

    return (device, config, input_stream)
}

fn start_speaker(ctx: &net_util::Context, config: rodio::SupportedStreamConfig) {
    println!("\nstarting speaker...");
    let device = rodio_util::find_device_by_name(
        cpal::default_host().output_devices().unwrap(),
        "sysdefault:CARD=X",
    ).unwrap();
    let (_os, output_stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    let sink = rodio::Sink::try_new(&output_stream_handle).unwrap();
    let mut buf = [0u8; 16_384*2];
    println!("starting loop");
    loop {
        let (amt, _src) = ctx.recv_socket.recv_from(&mut buf)
            .expect("Failed to read socket");
        if !gpio_util::is_call_active() {
            continue;
        }

        let ptr = buf.as_ptr() as *const i16;
        let len = amt / std::mem::size_of::<i16>();
        let samples: &[i16] = unsafe { std::slice::from_raw_parts(ptr, len) };

        let source = rodio::buffer::SamplesBuffer::new(
            config.channels(),
            config.sample_rate().0,
            samples.clone(),
        );
        sink.append(source);
    }
}
