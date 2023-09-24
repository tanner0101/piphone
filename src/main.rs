mod config;
mod poormans;
mod gpio_util;
mod rodio_util;
mod pcm;

use std::{self, net};
use std::io;
use biquad::ToHertz;
use config::Config;
use rodio::{cpal, DeviceTrait};
use rodio::cpal::traits::{HostTrait, StreamTrait};

fn main() -> io::Result<()> {
    let cfg = Config::new();

    // bind to random local port for sending
    // remote_addr is passed in socket send_to
    let remote_addr = format!("{}:{}", cfg.peer_addr, cfg.port);
    let send_socket = net::UdpSocket::bind("0.0.0.0:0")
        .expect("Failed to bind send socket");

    // bind to configured port for receiving
    let local_addr = format!("0.0.0.0:{}", cfg.port);
    let recv_socket = net::UdpSocket::bind(local_addr)
        .expect("Failed to bind recv socket");

    // start microphone and speaker
    let (_device, mic_config, _stream) = start_microphone(send_socket, remote_addr);
    start_speaker(recv_socket, mic_config);

    return io::Result::Ok(());
}

fn start_microphone(sock: net::UdpSocket, addr: String) -> (rodio::Device, rodio::SupportedStreamConfig, cpal::Stream) {
    println!("\nstarting microphone...");
    let host = cpal::default_host();
    let device = rodio_util::find_device_by_name(
        host.input_devices().unwrap(),
        "sysdefault:CARD=X",
    ).unwrap();
    let config = device.default_input_config().unwrap();
    let mut filter = poormans::Filter::new(config.sample_rate().0.hz());
    let input_stream = device.build_input_stream(
        &config.clone().into(),
        move |_data: &[i16], _: &cpal::InputCallbackInfo| {
            if !gpio_util::is_call_active() {
                return;
            }
            let data: Vec<i16> = _data.iter().map(|x| filter.run(*x)).collect();
            sock.send_to(pcm::to_buf(&data), &addr)
                .expect("failed to send");
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

fn start_speaker(sock: net::UdpSocket, mic_config: rodio::SupportedStreamConfig) {
    println!("\nstarting speaker...");
    let device = rodio_util::find_device_by_name(
        cpal::default_host().output_devices().unwrap(),
        "sysdefault:CARD=X",
    ).unwrap();
    let (_os, output_stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    let sink = rodio::Sink::try_new(&output_stream_handle).unwrap();
    let mut buf = [0u8; 16_384*2];
    loop {
        let (len, _src) = sock.recv_from(&mut buf)
            .expect("failed to read");
        if !gpio_util::is_call_active() {
            continue;
        }

        let source = rodio::buffer::SamplesBuffer::new(
            mic_config.channels(),
            mic_config.sample_rate().0,
            pcm::from_buf(&buf, len),
        );
        sink.append(source);
    }
}
