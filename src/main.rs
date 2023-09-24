mod config;
mod poormans;
mod gpio_util;
mod rodio_util;
mod pcm;

use std::{self, net};
use biquad::ToHertz;
use config::Config;
use rodio::{cpal, DeviceTrait};
use rodio::cpal::traits::{HostTrait, StreamTrait};

fn main() {
    let cfg = Config::new();
    let (mic_cfg, _mic_stream) = start_microphone(&cfg);
    start_speaker(mic_cfg, &cfg);
}

fn start_microphone(cfg: &Config) -> (rodio::SupportedStreamConfig, cpal::Stream) {
    println!("\nstarting microphone...");

    // bind to random local port for sending.
    // remote_addr is passed in socket send_to.
    let remote_addr = format!("{}:{}", cfg.peer_addr, cfg.port);
    let send_socket = net::UdpSocket::bind("0.0.0.0:0")
        .expect("Failed to bind send socket");

    let device = rodio_util::find_device_by_name(
        cpal::default_host().input_devices().unwrap(),
        &cfg.input_device_name,
    ).unwrap();
    let mic_cfg = device.default_input_config().unwrap();
    let mut filter = poormans::Filter::new(mic_cfg.sample_rate().0.hz());
    let mic_stream = device.build_input_stream(
        &mic_cfg.clone().into(),
        move |_data: &[i16], _: &cpal::InputCallbackInfo| {
            if !gpio_util::is_call_active() {
                return;
            }
            let data: Vec<i16> = _data.iter().map(|x| filter.run(*x)).collect();
            send_socket.send_to(pcm::to_buf(&data), &remote_addr)
                .expect("failed to send");
        },
        move |err| {
            eprintln!("an error occurred on stream: {}", err);
        },
        Option::None,
    )
        .unwrap();

    mic_stream
        .play()
        .unwrap();

    return (mic_cfg, mic_stream)
}

fn start_speaker(mic_cfg: rodio::SupportedStreamConfig, cfg: &Config) {
    println!("\nstarting speaker...");

    let local_addr = format!("0.0.0.0:{}", cfg.port);
    let recv_socket = net::UdpSocket::bind(local_addr)
        .expect("Failed to bind recv socket");

    let device = rodio_util::find_device_by_name(
        cpal::default_host().output_devices().unwrap(),
        &cfg.output_device_name,
    ).unwrap();
    let (_os, output_stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    let sink = rodio::Sink::try_new(&output_stream_handle).unwrap();
    let mut buf = [0u8; 16_384*2];
    loop {
        let (len, _src) = recv_socket.recv_from(&mut buf)
            .expect("failed to read");
        if !gpio_util::is_call_active() {
            continue;
        }

        let source = rodio::buffer::SamplesBuffer::new(
            mic_cfg.channels(),
            mic_cfg.sample_rate().0,
            pcm::from_buf(&buf, len),
        );
        sink.append(source);
    }
}
