mod call_state_machine;
mod config;
mod gpio_util;
mod pcm;
mod poormans;
mod rodio_util;

use biquad::ToHertz;
use config::Config;
use rodio::cpal::traits::{HostTrait, StreamTrait};
use rodio::{cpal, DeviceTrait};
use std::{self, net};

use std::sync::{Arc, RwLock};

use crate::call_state_machine::{Call, CallState, CallSwitchEdge, PacketType};

fn main() {
    let cfg = Config::new();
    let call = Arc::new(RwLock::new(Call::new()));
    println!("connecting to {}:{}...", cfg.peer_addr, cfg.port);
    let (mic_cfg, _mic_stream) = start_microphone(&cfg, Arc::clone(&call));
    start_speaker(Arc::clone(&call), mic_cfg, &cfg);
}

fn start_microphone(
    cfg: &Config,
    call: Arc<RwLock<Call>>,
) -> (rodio::SupportedStreamConfig, cpal::Stream) {
    println!("\nstarting microphone...");

    // bind to random local port for sending.
    // remote_addr is passed in socket send_to.
    let remote_addr = format!("{}:{}", cfg.peer_addr, cfg.port);
    let send_socket = net::UdpSocket::bind("0.0.0.0:0").expect("Failed to bind send socket");

    let device = rodio_util::find_device_by_name(
        cpal::default_host().input_devices().unwrap(),
        &cfg.headset_in_device,
    )
    .unwrap();
    let cfg_wrapper = cfg.clone();
    let mic_cfg = device.default_input_config().unwrap();
    let mut filter = poormans::Filter::new(mic_cfg.sample_rate().0.hz());
    let mic_stream = device
        .build_input_stream(
            &mic_cfg.clone().into(),
            move |_data: &[i16], _: &cpal::InputCallbackInfo| {
                let call_guard = call.read().unwrap();
                let call_state = call_guard.state.clone();

                println!("Call state in microphone: {:?}", call_state);

                if cfg_wrapper.uses_gpio && !gpio_util::is_call_active() {
                    return;
                }
                let data: Vec<i16> = _data.iter().map(|x| filter.run(*x)).collect();
                send_socket
                    .send_to(pcm::to_buf(&data), &remote_addr)
                    .expect("failed to send");
            },
            move |err| {
                eprintln!("an error occurred on stream: {}", err);
            },
            Option::None,
        )
        .unwrap();

    mic_stream.play().unwrap();

    return (mic_cfg, mic_stream);
}

fn start_speaker(call: Arc<RwLock<Call>>, mic_cfg: rodio::SupportedStreamConfig, cfg: &Config) {
    println!("\nstarting speaker...");

    let local_addr = format!("0.0.0.0:{}", cfg.port);
    let recv_socket = net::UdpSocket::bind(local_addr).expect("Failed to bind recv socket");

    let device = rodio_util::find_device_by_name(
        cpal::default_host().output_devices().unwrap(),
        &cfg.headset_out_device,
    )
    .unwrap();
    let (_os, output_stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    let sink = rodio::Sink::try_new(&output_stream_handle).unwrap();
    let mut buf = [0u8; 16_384 * 2];
    loop {
        let (len, _src) = recv_socket.recv_from(&mut buf).expect("failed to read");

        let call_switch = CallSwitchEdge::NoEdge;
        let packet_type: Option<PacketType> = None;

        // Call dispatch with new packet header:
        let mut call_guard = call.write().unwrap();
        call_guard.dispatch(&packet_type, &call_switch);
        let call_state: CallState = call_guard.state.clone();

        if cfg.uses_gpio && !gpio_util::is_call_active() {
            continue;
        }

        match call_state {
            CallState::InProgressCall => {
                let source = rodio::buffer::SamplesBuffer::new(
                    mic_cfg.channels(),
                    mic_cfg.sample_rate().0,
                    pcm::from_buf(&buf, len),
                );
                sink.append(source);
            }
            CallState::IncomingCall => {
                //ring
            }

            CallState::OutgoingCall => {
                //earpeice ring
            }

            CallState::Idle => {
                // do nothing
            }
        }
    }
}
